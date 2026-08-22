use iced::Color;

use crate::domain::{HexBounds, layer_inner::LayerInnerImpl};

use super::LayerKind;

/// An entirely empty layer that is used to draw the hex grid overlay.
#[derive(Debug, Clone, Copy)]
pub struct HexGridOverlay {
    colour: Color,
}

impl HexGridOverlay {
    pub fn new(colour: Color) -> Self {
        Self { colour }
    }

    pub fn new_dark() -> Self {
        Self {
            colour: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.1,
            },
        }
    }

    pub fn new_light() -> Self {
        Self {
            colour: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.1,
            },
        }
    }
}

impl LayerInnerImpl for HexGridOverlay {
    fn kind(&self) -> LayerKind {
        LayerKind::Utility
    }

    fn bounds(&self, _hex_size: f32) -> Option<iced::Rectangle> {
        None
    }

    fn draw(&self, renderer: &mut dyn crate::domain::RenderTarget) {
        let bounds = renderer.get_bounds();
        let hexes = HexBounds::from_rect(bounds).into_hexes();

        for coord in hexes {
            let point = renderer.hex_to_point(&coord);
            renderer.stroke_polygon(&point, self.colour);
        }
    }
}
