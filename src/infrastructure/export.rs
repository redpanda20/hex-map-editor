use iced::Rectangle;
use image::{ImageBuffer, Rgba};

use crate::domain::{HexBounds, Scene};

const HEX_SIZE: f32 = 100.0;

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

pub fn export_png(layers: &Scene) -> Vec<u8> {
    // Determine bounding box of all painted tiles

    let hex_bounds = layers
        .get_visible_layers()
        .iter()
        .filter_map(|inner| inner.get_bounds())
        .reduce(|acc, e| HexBounds::union(&acc, &e));

    let bounding_box = layers
        .get_visible_layers()
        .iter()
        .filter_map(|inner| inner.get_bounding_box(HEX_SIZE))
        .reduce(|acc, e| Rectangle::union(&acc, &e));

    // Create placeholder image if nothing has been drawn
    let (Some(hex_bounds), Some(bounding_box)) = (hex_bounds, bounding_box) else {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(256, 256);
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .ok();
        return out;
    };

    // Create image background to fit content
    let bounds = bounding_box.expand(2.0 * HEX_SIZE);
    let img_w = bounds.width.ceil() as u32;
    let img_h = bounds.height.ceil() as u32;

    let mut buf: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(img_w, img_h);

    // Fill with transparent background
    for p in buf.pixels_mut() {
        *p = Rgba([0, 0, 0, 0]);
    }

    // let hex_bounds = HexBounds::from_rect(bounds);

    // Draw all layers
    for layer in layers.get_visible_layers() {
        let coords = hex_bounds.into_hexes();
        layer.draw(&mut buf, coords, |buf, tile, colour| {
            let hex = tile.to_cartesian() * HEX_SIZE;
            let x = hex.x - bounds.x;
            let y = hex.y - bounds.y;
            let verts: Vec<(f32, f32)> =
                hex_vertices_f(x, y).iter().map(|(x, y)| (*x, *y)).collect();
            fill_polygon(buf, &verts, colour.into_rgba8());
        });
    }

    let mut out = Vec::new();
    buf.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding failed");
    out
}
