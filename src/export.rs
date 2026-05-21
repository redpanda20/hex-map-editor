// src/export.rs — PNG and PDF export
//
// PNG export renders all visible layers to a pixel buffer using the same
// hex geometry as the canvas, then encodes to PNG bytes.
//
// PDF export places filled polygons for each hex cell using printpdf.

use image::{ImageBuffer, Rgba};

use crate::state::{HexCoord, Layer};

// ---------------------------------------------------------------------------
// Shared geometry helper
// ---------------------------------------------------------------------------

fn hex_vertices_f(cx: f32, cy: f32, size: f32) -> [(f32, f32); 6] {
    std::array::from_fn(|i| {
        let angle_rad = (60.0 * i as f32).to_radians();
        (cx + size * angle_rad.cos(), cy + size * angle_rad.sin())
    })
}

fn fill_polygon(buf: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, vertices: &[(f32, f32)], color: [u8; 4]) {
    let width = buf.width() as f32;
    let height = buf.height() as f32;

    // Axis-aligned bounding box of the polygon.
    let xs: Vec<f32> = vertices.iter().map(|(x, _)| *x).collect();
    let ys: Vec<f32> = vertices.iter().map(|(_, y)| *y).collect();
    let xmin = xs.iter().cloned().fold(f32::INFINITY, f32::min).max(0.0) as u32;
    let xmax = xs
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
        .min(width - 1.0) as u32;
    let ymin = ys.iter().cloned().fold(f32::INFINITY, f32::min).max(0.0) as u32;
    let ymax = ys
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
        .min(height - 1.0) as u32;

    for py in ymin..=ymax {
        for px in xmin..=xmax {
            if point_in_polygon(px as f32 + 0.5, py as f32 + 0.5, vertices) {
                // Alpha-blend over existing pixel.
                let existing = buf.get_pixel(px, py);
                let bg = [existing[0], existing[1], existing[2], existing[3]];
                let out = alpha_blend(bg, color);
                buf.put_pixel(px, py, Rgba(out));
            }
        }
    }
}

fn point_in_polygon(x: f32, y: f32, verts: &[(f32, f32)]) -> bool {
    let n = verts.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = verts[i];
        let (xj, yj) = verts[j];
        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn alpha_blend(bg: [u8; 4], fg: [u8; 4]) -> [u8; 4] {
    let fa = fg[3] as f32 / 255.0;
    let ba = bg[3] as f32 / 255.0;
    let out_a = fa + ba * (1.0 - fa);
    if out_a == 0.0 {
        return [0, 0, 0, 0];
    }
    let blend =
        |fc: u8, bc: u8| -> u8 { ((fc as f32 * fa + bc as f32 * ba * (1.0 - fa)) / out_a) as u8 };
    [
        blend(fg[0], bg[0]),
        blend(fg[1], bg[1]),
        blend(fg[2], bg[2]),
        (out_a * 255.0) as u8,
    ]
}

pub fn export_png(layers: &Vec<Layer>, hex_size: f32, padding: f32) -> Vec<u8> {
    let size = hex_size;

    // Determine bounding box of all painted tiles.
    let all_coords: Vec<HexCoord> = layers
        .iter()
        .filter(|l| l.visible)
        .flat_map(|l| l.tiles.iter().copied())
        .collect();

    if all_coords.is_empty() {
        // Return a small placeholder image.
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(256, 256);
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .ok();
        return out;
    }

    let pixels: Vec<(f32, f32)> = all_coords.iter().map(|c| c.to_pixel(size)).collect();
    let xmin = pixels
        .iter()
        .map(|(x, _)| x)
        .cloned()
        .fold(f32::INFINITY, f32::min)
        - size
        - padding;
    let ymin = pixels
        .iter()
        .map(|(_, y)| y)
        .cloned()
        .fold(f32::INFINITY, f32::min)
        - size
        - padding;
    let xmax = pixels
        .iter()
        .map(|(x, _)| x)
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
        + size
        + padding;
    let ymax = pixels
        .iter()
        .map(|(_, y)| y)
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
        + size
        + padding;

    let img_w = (xmax - xmin).ceil() as u32;
    let img_h = (ymax - ymin).ceil() as u32;

    let mut buf: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(img_w, img_h);

    // Fill dark background.
    for p in buf.pixels_mut() {
        *p = Rgba([30, 32, 40, 255]);
    }

    // Draw grid outlines first.
    // (Skipped for brevity — add if desired.)

    // Draw each layer bottom → top.
    for layer in layers.iter() {
        if !layer.visible {
            continue;
        }
        for tile in layer.tiles.iter() {
            let (cx, cy) = tile.to_pixel(size);
            let sx = cx - xmin;
            let sy = cy - ymin;
            let verts: Vec<(f32, f32)> = hex_vertices_f(sx, sy, size)
                .iter()
                .map(|(x, y)| (*x, *y))
                .collect();
            let color = layer.color.into_rgba8();
            fill_polygon(&mut buf, &verts, color);
        }
    }

    let mut out = Vec::new();
    buf.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding failed");
    out
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_bytes_as(bytes: &[u8], default_name: &str, _mime: &str) -> String {
    use rfd::FileDialog;

    let extension = default_name.rsplit('.').next().unwrap_or("bin");
    let path = FileDialog::new()
        .add_filter(extension, &[extension])
        .set_file_name(default_name)
        .save_file();

    match path {
        Some(p) => match std::fs::write(&p, bytes) {
            Ok(_) => format!("Saved to {}", p.display()),
            Err(e) => format!("Save failed: {}", e),
        },
        None => "Export cancelled.".into(),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn save_bytes_as(bytes: &[u8], filename: &str, mime: &str) -> String {
    use js_sys::{Array, Uint8Array};
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let uint8_array = Uint8Array::from(bytes);
    let array = Array::new();
    array.push(&uint8_array);

    let mut opts = BlobPropertyBag::new();
    opts.type_(mime);
    let blob =
        Blob::new_with_u8_array_sequence_and_options(&array, &opts).expect("blob creation failed");

    let url = Url::create_object_url_with_blob(&blob).expect("URL creation failed");

    let a: HtmlAnchorElement = document
        .create_element("a")
        .expect("createElement failed")
        .dyn_into()
        .expect("cast failed");

    a.set_href(&url);
    a.set_download(filename);
    a.click();

    Url::revoke_object_url(&url).ok();

    format!("Downloading {filename}…")
}
