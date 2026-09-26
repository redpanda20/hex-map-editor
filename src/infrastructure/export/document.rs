//! Port for interacting with pdf documents.

use std::collections::{BTreeSet, HashMap};

use iced::advanced::image::Handle;
use pdf_writer::{Filter, Finish, Name, Pdf, Ref, TextStr};

use crate::domain::{assets::AssetStore, id::ImageId};

struct EmbeddedImage {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
    /// `None` when the image is fully opaque, so no soft mask is needed.
    alpha: Option<Vec<u8>>,
}

#[derive(Default)]
pub(super) struct PdfDocument {
    images: Vec<EmbeddedImage>,
    image_index: HashMap<ImageId, usize>,
    alphas: BTreeSet<u8>,
}

impl PdfDocument {
    pub(super) fn alpha_key(&mut self, alpha: f32) -> u8 {
        let alpha = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
        self.alphas.insert(alpha);
        alpha
    }

    /// Embeds the image on first use and returns its index either way.
    /// `None` only if the asset can't be read as RGBA pixels.
    pub(super) fn image_key(&mut self, assets: &AssetStore, id: ImageId) -> Option<usize> {
        if let Some(index) = self.image_index.get(&id) {
            return Some(*index);
        }

        let image = embed_image(assets, id)?;
        self.images.push(image);
        let index = self.images.len() - 1;
        self.image_index.insert(id, index);
        Some(index)
    }
}

/// Copies the source pixels of an image, separating colour and alpha.
fn embed_image(assets: &AssetStore, id: ImageId) -> Option<EmbeddedImage> {
    let Handle::Rgba {
        width,
        height,
        pixels,
        ..
    } = assets.image_data(id)?
    else {
        return None;
    };

    let pixel_count = (*width as usize) * (*height as usize);
    if pixel_count == 0 || pixels.len() < pixel_count * 4 {
        return None;
    }

    let mut rgb = Vec::with_capacity(pixel_count * 3);
    let mut alpha = Vec::with_capacity(pixel_count);
    for px in pixels[..pixel_count * 4].chunks_exact(4) {
        rgb.extend_from_slice(&px[..3]);
        alpha.push(px[3]);
    }

    let has_transparency = alpha.iter().any(|a| *a != 255);

    Some(EmbeddedImage {
        width: *width,
        height: *height,
        rgb,
        alpha: has_transparency.then_some(alpha),
    })
}

fn deflate(data: &[u8]) -> Vec<u8> {
    miniz_oxide::deflate::compress_to_vec_zlib(data, 6)
}

/// A resource name such as `A128` or `Im0`, owned so it can outlive a format call.
pub(super) struct ResourceName(String);

impl ResourceName {
    pub(super) fn as_name(&self) -> Name<'_> {
        Name(self.0.as_bytes())
    }
}

pub(super) fn alpha_name(alpha: u8) -> ResourceName {
    ResourceName(format!("A{alpha}"))
}

pub(super) fn image_name(index: usize) -> ResourceName {
    ResourceName(format!("Im{index}"))
}

/// Construct a PDF document.
///
/// `pages` are a tuple of content bytes & images, per page.
pub(super) fn assemble_pdf(
    doc: PdfDocument,
    pages: Vec<(Vec<u8>, Vec<usize>)>,
    page_size_pt: (f32, f32),
    title: &str,
) -> Vec<u8> {
    let mut next = 1;
    let mut alloc = move || {
        let r = Ref::new(next);
        next += 1;
        r
    };

    let catalog_id = alloc();
    let pages_id = alloc();
    let font_id = alloc();
    let info_id = alloc();

    let page_ids: Vec<Ref> = pages.iter().map(|_| alloc()).collect();
    let content_ids: Vec<Ref> = pages.iter().map(|_| alloc()).collect();
    let alpha_ids: Vec<(u8, Ref)> = doc.alphas.iter().map(|a| (*a, alloc())).collect();
    let image_ids: Vec<(Ref, Option<Ref>)> = doc
        .images
        .iter()
        .map(|img| (alloc(), img.alpha.as_ref().map(|_| alloc())))
        .collect();

    let mut pdf = Pdf::new();
    pdf.catalog(catalog_id).pages(pages_id);
    pdf.pages(pages_id)
        .kids(page_ids.iter().copied())
        .count(page_ids.len() as i32);
    pdf.document_info(info_id)
        .title(TextStr(title))
        .creator(TextStr("HexMap Editor"));

    let (page_w, page_h) = page_size_pt;

    for (i, (page_id, (_, used_images))) in page_ids.iter().zip(&pages).enumerate() {
        let mut page = pdf.page(*page_id);
        page.media_box(pdf_writer::Rect::new(0.0, 0.0, page_w, page_h))
            .parent(pages_id)
            .contents(content_ids[i]);

        let mut resources = page.resources();
        resources.fonts().pair(Name(b"F1"), font_id);
        {
            let mut states = resources.ext_g_states();
            for (alpha, id) in &alpha_ids {
                states.pair(alpha_name(*alpha).as_name(), *id);
            }
        }
        {
            let mut xobjects = resources.x_objects();
            for &index in used_images {
                xobjects.pair(image_name(index).as_name(), image_ids[index].0);
            }
        }
    }

    pdf.type1_font(font_id).base_font(Name(b"Helvetica"));

    for (alpha, id) in &alpha_ids {
        let a = *alpha as f32 / 255.0;
        pdf.ext_graphics(*id)
            .non_stroking_alpha(a)
            .stroking_alpha(a);
    }

    for (image, (id, mask_id)) in doc.images.iter().zip(&image_ids) {
        let (w, h) = (image.width as i32, image.height as i32);

        let data = deflate(&image.rgb);
        let mut xobject = pdf.image_xobject(*id, &data);
        xobject.filter(Filter::FlateDecode);
        xobject.width(w).height(h);
        xobject.color_space().device_rgb();
        xobject.bits_per_component(8);
        if let Some(mask_id) = mask_id {
            xobject.s_mask(*mask_id);
        }
        xobject.finish();

        if let (Some(mask_id), Some(alpha)) = (mask_id, &image.alpha) {
            let data = deflate(alpha);
            let mut mask = pdf.image_xobject(*mask_id, &data);
            mask.filter(Filter::FlateDecode);
            mask.width(w).height(h);
            mask.color_space().device_gray();
            mask.bits_per_component(8);
            mask.finish();
        }
    }

    for (i, (content, _)) in pages.iter().enumerate() {
        let data = deflate(content);
        pdf.stream(content_ids[i], &data)
            .filter(Filter::FlateDecode);
    }

    pdf.finish()
}
