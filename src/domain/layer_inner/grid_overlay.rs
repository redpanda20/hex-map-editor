use iced::Color;

use crate::domain::{HexBounds, layer_inner::LayerInnerImpl};

use super::LayerKind;

/// An entirely empty layer that is used to draw the hex grid overlay.
#[derive(Debug, Clone, Copy)]
pub struct HexGridOverlay;

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
            let colour = Color::from_rgba(1.0, 1.0, 1.0, 0.1);
            renderer.stroke_polygon(&point, colour);
        }
    }
}
