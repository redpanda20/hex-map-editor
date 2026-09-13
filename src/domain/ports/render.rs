use iced::{Color, Point, Rectangle};

use crate::domain::{HexCoord, id::ImageId};

/// Provides methods to draw to a given target
///
/// Used to delegate rendering to each LayerInner
/// Implemented by HexCanvas, ExportPng
pub trait RenderTarget {
    fn hex_to_point(&self, coord: &HexCoord) -> Point;
    fn get_bounds(&self) -> Rectangle;

    fn fill_polygon(&mut self, point: &Point, fill: Color);

    fn stroke_polygon(&mut self, point: &Point, colour: Color, stroke_width: f32);

    fn draw_image(&mut self, bounds: Rectangle, image: ImageId, opacity: f32);

    // fn draw_text(&mut self, point: Point, size: Size, text: &str);
}

/// A mock [`RenderTarget`].
/// Records every draw call made to it instead of drawing anything.
/// Can stand in `HexCanvas` or `ExportPng` targets in tests.
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct MockRenderer {
    pub bounds: Rectangle,
    pub fills: Vec<(Point, Color)>,
    pub strokes: Vec<(Point, Color, f32)>,
    pub images: Vec<(Rectangle, ImageId, f32)>,
}

#[cfg(test)]
impl MockRenderer {
    /// A renderer whose viewport is `bounds`, with no recorded calls yet.
    pub fn with_bounds(bounds: Rectangle) -> Self {
        Self {
            bounds,
            ..Default::default()
        }
    }
}
#[cfg(test)]
impl RenderTarget for MockRenderer {
    fn hex_to_point(&self, coord: &HexCoord) -> Point {
        let v = coord.to_cartesian();
        Point::new(v.x, v.y)
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn fill_polygon(&mut self, point: &Point, fill: Color) {
        self.fills.push((*point, fill));
    }

    fn stroke_polygon(&mut self, point: &Point, colour: Color, stroke_width: f32) {
        self.strokes.push((*point, colour, stroke_width));
    }

    fn draw_image(&mut self, bounds: Rectangle, image: ImageId, opacity: f32) {
        self.images.push((bounds, image, opacity));
    }
}
