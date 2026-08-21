use iced::{Color, Point};

use crate::domain::HexCoord;

/// Provides methods to draw to a given target
///
/// Used to delegate rendering to each LayerInner
/// Implemented by ~~HexCanvas~~, ~~ExportPng~~
pub trait RenderTarget {
    type Target;

    fn hex_to_point(coord: &HexCoord) -> Point;

    fn fill_polygon(target: &mut Self::Target, point: &Point, fill: Color);

    // fn stroke_polygon(target: &mut Self::Target, point: &Point, stroke: Stroke);

    // fn draw_image(target: &mut Self::Target, point: Point, size: Size, image: &Image, opacity: f32);

    // fn draw_text(target: &mut Self::Target, point: Point, size: Size, text: &str);
}
