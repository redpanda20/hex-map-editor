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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::render::MockRenderer;

    #[test]
    fn dark_variant_is_a_translucent_black() {
        let overlay = HexGridOverlay::new_dark(1.0);
        assert_eq!(
            overlay.colour,
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.1
            }
        );
    }

    #[test]
    fn light_variant_is_a_translucent_white() {
        let overlay = HexGridOverlay::new_light(1.0);
        assert_eq!(
            overlay.colour,
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.1
            }
        );
    }

    #[test]
    fn bounds_is_always_none() {
        let overlay = HexGridOverlay::new_dark(1.0);
        assert!(overlay.bounds(16.0).is_none());
    }

    #[test]
    fn draw_strokes_every_visible_hex_with_the_configured_style() {
        let overlay = HexGridOverlay::new(Color::WHITE, 2.5);
        let bounds = HexBounds::new(-1, 1, -1, 1).into_rect();
        let expected = HexBounds::from_rect(bounds).into_hexes().count();

        let mut renderer = MockRenderer::with_bounds(bounds);
        overlay.draw(&mut renderer);

        assert_eq!(renderer.strokes.len(), expected);
        assert!(renderer.fills.is_empty(), "overlay should never fill");
        for (_, colour, width) in &renderer.strokes {
            assert_eq!(*colour, Color::WHITE);
            assert_eq!(*width, 2.5);
        }
    }
}
