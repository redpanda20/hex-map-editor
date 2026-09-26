mod document;
mod pdf_target;
mod png_export;

use iced::{Point, Rectangle};
use image::{ImageBuffer, Rgba};

use crate::domain::{
    Scene,
    layer::overlay::HexGridOverlay,
    print::{PageGrid, PrintSettings},
};
use document::{PdfDocument, assemble_pdf};
use pdf_target::PdfPageTarget;
use png_export::PngRenderTarget;

const EXPORT_HEX_SIZE: f32 = 100.0;

/// The file formats a scene can be exported to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    Png,
    Pdf,
}

impl ExportFormat {
    pub fn name(self) -> &'static str {
        match self {
            ExportFormat::Png => "PNG",
            ExportFormat::Pdf => "PDF",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            ExportFormat::Png => "png",
            ExportFormat::Pdf => "pdf",
        }
    }
}

/// Margin around the map content, in hexes (matches the PNG export).
const EXPORT_MARGIN_HEXES: f32 = 2.0;

/// Page size, in hexes, used when there is nothing to export.
const EMPTY_PAGE_HEXES: f32 = 8.0;

pub fn export_png(scene: &Scene) -> Vec<u8> {
    let bounding_box = scene
        .get_visible_layers()
        .iter()
        .filter_map(|inner| inner.bounds(EXPORT_HEX_SIZE))
        .reduce(|acc, bounds| Rectangle::union(&acc, &bounds));

    let Some(bounding_box) = bounding_box else {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(256, 256);

        let mut out = Vec::new();

        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .expect("PNG encoding failed");

        return out;
    };

    let bounds = bounding_box.expand(2.0 * EXPORT_HEX_SIZE);

    let width = bounds.width.ceil() as u32;
    let height = bounds.height.ceil() as u32;

    let mut image = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));

    let mut target = PngRenderTarget::new(&mut image, bounds, &scene.assets);
    let mut layers = scene.get_visible_layers();
    let overlay = HexGridOverlay::new_dark(1.5);
    layers.push(&overlay);

    for layer in layers {
        layer.draw(&mut target);
    }

    let mut out = Vec::new();

    image
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding failed");

    out
}

/// Calculate the area the scene will cover.
fn pdf_export_bounds(scene: &Scene) -> Rectangle {
    let bounding_box = scene
        .get_visible_layers()
        .iter()
        .filter_map(|inner| inner.bounds(1.0))
        .reduce(|acc, bounds| Rectangle::union(&acc, &bounds));

    match bounding_box {
        Some(bounding_box) => bounding_box.expand(EXPORT_MARGIN_HEXES),
        None => Rectangle::new(
            iced::Point::new(-EMPTY_PAGE_HEXES / 2.0, -EMPTY_PAGE_HEXES / 2.0),
            iced::Size::new(EMPTY_PAGE_HEXES, EMPTY_PAGE_HEXES),
        ),
    }
}

/// Calculate the number of pages required to draw
/// a `Scene` with the given `PrintSettings`.
pub fn pdf_page_count(scene: &Scene, settings: PrintSettings) -> PageGrid {
    settings.page_grid(pdf_export_bounds(scene))
}

/// Exports the visible layers as a vector PDF drawn to scale, tiled across
/// as many pages as the map needs at `settings`.
pub fn export_pdf(scene: &Scene, settings: PrintSettings) -> Vec<u8> {
    let bounds = pdf_export_bounds(scene);

    let grid = settings.page_grid(bounds);
    let tile = settings.tile_size();
    let page_size_pt = settings.page_size.size_points();

    const TITLE: &str = "HexMap";

    let mut doc = PdfDocument::default();
    let mut pages = Vec::new();

    for row in 0..grid.rows {
        for col in 0..grid.columns {
            let tile_bounds = Rectangle::new(
                Point::new(
                    bounds.x + col as f32 * tile.width,
                    bounds.y + row as f32 * tile.height,
                ),
                tile,
            );

            let mut target =
                PdfPageTarget::new(&mut doc, &scene.assets, tile_bounds, settings, page_size_pt);

            let mut layers = scene.get_visible_layers();
            let overlay = HexGridOverlay::new_dark(1.5);
            layers.push(&overlay);

            for layer in layers {
                layer.draw(&mut target);
            }

            pages.push(target.finish());
        }
    }

    assemble_pdf(doc, pages, page_size_pt, TITLE)
}
