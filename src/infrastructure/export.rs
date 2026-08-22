mod png_export;

use iced::{Point, Rectangle};
use image::{ImageBuffer, Rgba};

use crate::domain::{HexBounds, Scene};
use png_export::PngRenderTarget;

const EXPORT_HEX_SIZE: f32 = 100.0;

pub fn export_png(scene: &Scene) -> Vec<u8> {
    let hex_bounds = scene
        .get_visible_layers()
        .iter()
        .filter_map(|inner| inner.get_bounds())
        .reduce(|acc, bounds| HexBounds::union(&acc, &bounds));

    let bounding_box = scene
        .get_visible_layers()
        .iter()
        .filter_map(|inner| inner.get_bounding_box(EXPORT_HEX_SIZE))
        .reduce(|acc, bounds| Rectangle::union(&acc, &bounds));

    let (Some(hex_bounds), Some(bounding_box)) = (hex_bounds, bounding_box) else {
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

    let mut target = PngRenderTarget::new(&mut image, Point::new(bounds.x, bounds.y));

    for layer in scene.get_visible_layers() {
        layer.draw(&mut target, hex_bounds.into_hexes());
    }

    let mut out = Vec::new();

    image
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding failed");

    out
}
