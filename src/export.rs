// src/export.rs — PNG and PDF export
//
// PNG export renders all visible layers to a pixel buffer using the same
// hex geometry as the canvas, then encodes to PNG bytes.
//
// PDF export places filled polygons for each hex cell using printpdf.

use image::{ImageBuffer, Rgba};

use crate::state::{HexCoord, Layer};

const RENDER_SCALE: f32 = 2.0;
const HEX_SIZE: f32 = crate::state::HEX_SIZE * RENDER_SCALE;

// ---------------------------------------------------------------------------
// Shared geometry helper
// ---------------------------------------------------------------------------

fn hex_vertices_f(cx: f32, cy: f32) -> [(f32, f32); 6] {
    std::array::from_fn(|i| {
        let angle_rad = (60.0 * i as f32).to_radians();
        (
            cx + HEX_SIZE * angle_rad.cos(),
            cy + HEX_SIZE * angle_rad.sin(),
        )
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
                // TODO: Blend layers with alpha channels

                buf.put_pixel(px, py, Rgba(color));
            }
        }
    }
}

fn point_in_polygon(x: f32, y: f32, verticies: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let mut j = verticies.len() - 1;
    for i in 0..verticies.len() {
        let (xi, yi) = verticies[i];
        let (xj, yj) = verticies[j];
        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub fn export_png(layers: &Vec<Layer>) -> Vec<u8> {
    // Determine bounding box of all painted tiles
    let all_coords: Vec<HexCoord> = layers
        .iter()
        .filter(|l| l.visible)
        .flat_map(|l| l.tiles.iter().copied())
        .collect();

    // Create placeholder image if nothing has been drawn
    if all_coords.is_empty() {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(256, 256);
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .ok();
        return out;
    }

    let pixels: Vec<(f32, f32)> = all_coords.iter().map(|c| c.to_pixel(HEX_SIZE)).collect();
    let xmin = pixels
        .iter()
        .map(|(x, _)| x)
        .cloned()
        .reduce(f32::min)
        .unwrap()
        - HEX_SIZE * 2.0;
    let ymin = pixels
        .iter()
        .map(|(_, y)| y)
        .cloned()
        .reduce(f32::min)
        .unwrap()
        - HEX_SIZE * 2.0;
    let xmax = pixels
        .iter()
        .cloned()
        .map(|(x, _)| x)
        .reduce(f32::max)
        .unwrap()
        + HEX_SIZE * 2.0;
    let ymax = pixels
        .iter()
        .cloned()
        .map(|(_, y)| y)
        .reduce(f32::max)
        .unwrap()
        + HEX_SIZE * 2.0;

    let img_w = (xmax - xmin).ceil() as u32;
    let img_h = (ymax - ymin).ceil() as u32;

    let mut buf: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(img_w, img_h);

    // Background
    for p in buf.pixels_mut() {
        *p = Rgba([30, 32, 40, 255]);
    }

    // Draw each layer bottom → top.
    for layer in layers.iter() {
        if !layer.visible {
            continue;
        }
        for tile in layer.tiles.iter() {
            let (cx, cy) = tile.to_pixel(HEX_SIZE);
            let x = cx - xmin;
            let y = cy - ymin;
            let verts: Vec<(f32, f32)> =
                hex_vertices_f(x, y).iter().map(|(x, y)| (*x, *y)).collect();
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
