use iced::{Color, Point, Rectangle};

use crate::domain::HexCoord;

/// Provides methods to draw to a given target
///
/// Used to delegate rendering to each LayerInner
/// Implemented by HexCanvas, ExportPng
pub trait RenderTarget {
    fn hex_to_point(&self, coord: &HexCoord) -> Point;
    fn get_bounds(&self) -> Rectangle;

    fn fill_polygon(&mut self, point: &Point, fill: Color);

    fn stroke_polygon(&mut self, point: &Point, colour: Color);

    // fn draw_image(&mut self, point: Point, size: Size, image: &Image, opacity: f32);

    // fn draw_text(&mut self, point: Point, size: Size, text: &str);
}
