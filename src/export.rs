use iced::{Padding, Rectangle, Task};
use image::{EncodableLayout, ImageBuffer, Rgba};

use crate::{
    app::Message,
    state::{Layers, SparseTiles},
};

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

/// Creates bounding box
/// returns xmin, xmax, ymin, ymax
fn get_bounds(source: &Layers) -> iced::Rectangle {
    let mut xmin = f32::MAX;
    let mut xmax = f32::MIN;
    let mut ymin = f32::MAX;
    let mut ymax = f32::MIN;

    for layer in source.get_visible_layers() {
        match layer {
            crate::state::LayerInner::Tiles(sparse_tiles)
            | crate::state::LayerInner::InvertedTiles(sparse_tiles) => {
                for tile in sparse_tiles.tiles.iter() {
                    let vec = tile.to_pixel(HEX_SIZE);
                    xmin = xmin.min(vec.x);
                    xmax = xmax.max(vec.x);
                    ymin = ymin.min(vec.y);
                    ymax = ymax.max(vec.y);
                }
            }
        }
    }

    iced::Rectangle {
        x: xmin,
        y: ymin,
        width: xmax - xmin,
        height: ymax - ymin,
    }
}

pub fn export_png(layers: &Layers) -> Vec<u8> {
    // Determine bounding box of all painted tiles
    let bounds = layers
        .get_visible_layers()
        .iter()
        .filter_map(|inner| match inner {
            crate::state::LayerInner::Tiles(sparse_tiles) => sparse_tiles.bounding_box(HEX_SIZE),
            crate::state::LayerInner::InvertedTiles(sparse_tiles) => {
                sparse_tiles.bounding_box(HEX_SIZE)
            }
        })
        .reduce(|acc, e| Rectangle::union(&acc, &e));

    // Create placeholder image if nothing has been drawn
    let Some(bounds) = bounds else {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(256, 256);
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .ok();
        return out;
    };

    // Create image background to fit content
    let bounds = bounds.expand(HEX_SIZE * 2.0);
    let img_w = bounds.width.ceil() as u32;
    let img_h = bounds.height.ceil() as u32;

    let mut buf: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(img_w, img_h);

    // Background
    for p in buf.pixels_mut() {
        *p = Rgba([255, 255, 255, 255]);
    }

    // Draw each layer bottom → top.
    for layer in layers.get_visible_layers() {
        match layer {
            crate::state::LayerInner::Tiles(sparse_tiles)
            | crate::state::LayerInner::InvertedTiles(sparse_tiles) => {
                for tile in sparse_tiles.tiles.iter() {
                    let hex = tile.to_pixel(HEX_SIZE);
                    let x = hex.x - bounds.x;
                    let y = hex.y - bounds.y;
                    let verts: Vec<(f32, f32)> =
                        hex_vertices_f(x, y).iter().map(|(x, y)| (*x, *y)).collect();
                    let color = sparse_tiles.colour.into_rgba8();
                    fill_polygon(&mut buf, &verts, color);
                }
            }
        }
    }

    let mut out = Vec::new();
    buf.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding failed");
    out
}

pub fn save_bytes_async(bytes: Vec<u8>, default_name: &str) -> Task<Message> {
    use rfd::AsyncFileDialog;

    Task::future(
        AsyncFileDialog::new()
            .set_file_name(default_name)
            .set_title("Export to PNG")
            .save_file(),
    )
    .then(move |handle| {
        let inner_bytes = bytes.clone();
        match handle {
            Some(file_handle) => {
                Task::perform(write_future(file_handle, inner_bytes), Message::Exported)
            }
            None => Task::done(Message::ExportCancelled),
        }
    })
}

async fn write_future(handle: rfd::FileHandle, bytes: Vec<u8>) -> Result<(), String> {
    handle
        .write(bytes.as_bytes())
        .await
        .map_err(|err| err.to_string())
}
