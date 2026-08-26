use iced::Color;

use super::LayerInnerImpl;
use crate::domain::HexBounds;

/// An entirely empty layer that is used to draw the hex grid overlay.
#[derive(Debug, Clone, Copy)]
pub struct HexGridOverlay {
    colour: Color,
    stroke_width: f32,
}

impl HexGridOverlay {
    pub fn new(colour: Color, stroke_width: f32) -> Self {
        Self {
            colour,
            stroke_width,
        }
    }

    pub fn new_dark(stroke_width: f32) -> Self {
        let colour = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.1,
        };
        Self::new(colour, stroke_width)
    }

    pub fn new_light(stroke_width: f32) -> Self {
        let colour = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.1,
        };
        Self::new(colour, stroke_width)
    }
}

impl LayerInnerImpl for HexGridOverlay {
    fn bounds(&self, _hex_size: f32) -> Option<iced::Rectangle> {
        None
    }

    fn draw(&self, renderer: &mut dyn crate::domain::RenderTarget) {
        let bounds = renderer.get_bounds();
        let hexes = HexBounds::from_rect(bounds).into_hexes();

        for coord in hexes {
            let point = renderer.hex_to_point(&coord);
            renderer.stroke_polygon(&point, self.colour, self.stroke_width);
        }
    }
}
