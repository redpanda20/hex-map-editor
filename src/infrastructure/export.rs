mod png_export;

use iced::Rectangle;
use image::{ImageBuffer, Rgba};

use crate::domain::{Scene, layer::overlay::HexGridOverlay};
use png_export::PngRenderTarget;

const EXPORT_HEX_SIZE: f32 = 100.0;

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
