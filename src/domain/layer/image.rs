use iced::{Point, Rectangle, Size};

use crate::domain::{RenderTarget, id::ImageId, layer::LayerInnerImpl};

#[derive(Debug, Default, Clone)]
pub struct ImageLayer {
    pub image: Option<ImageId>,
    pub size: Size,
    pub position: Point,
    opacity: f32,
}

impl ImageLayer {
    pub fn new() -> Self {
        Self {
            image: None,
            size: Size::default(),
            position: Point::default(),
            opacity: 1.0,
        }
    }

    pub fn new_with(image: ImageId) -> Self {
        Self {
            image: Some(image),
            size: Size::default(),
            position: Point::default(),
            opacity: 1.0,
        }
    }

    pub fn get_opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn get_bounds(&self) -> Rectangle {
        Rectangle::new(self.position, self.size)
    }
}

impl LayerInnerImpl for ImageLayer {
    fn bounds(&self, hex_size: f32) -> Option<Rectangle> {
        // Technically this shouldn't be known,
        // exports are currently broken otherwise
        const EDITOR_HEX_SIZE: f32 = 16.0;
        let relative_size = hex_size / EDITOR_HEX_SIZE;
        Some(self.get_bounds() * relative_size)
    }

    fn draw(&self, renderer: &mut dyn RenderTarget) {
        if let Some(image) = self.image {
            renderer.draw_image(self.get_bounds(), image, self.opacity);
        }
    }
}

#[cfg(test)]
mod tests {
    use iced::Point;

    use super::*;
    use crate::domain::ports::MockRenderer;

    #[test]
    fn new_layer_has_no_image_and_full_opacity() {
        let layer = ImageLayer::new();
        assert_eq!(layer.image, None);
        assert_eq!(layer.get_opacity(), 1.0);
    }

    #[test]
    fn new_with_sets_the_image() {
        let id = ImageId::from_raw(1);
        let layer = ImageLayer::new_with(id);
        assert_eq!(layer.image, Some(id));
    }

    #[test]
    fn set_opacity_clamps_to_the_unit_range() {
        let mut layer = ImageLayer::new();
        layer.set_opacity(1.5);
        assert_eq!(layer.get_opacity(), 1.0);
        layer.set_opacity(-0.5);
        assert_eq!(layer.get_opacity(), 0.0);
        layer.set_opacity(0.5);
        assert_eq!(layer.get_opacity(), 0.5);
    }

    #[test]
    fn bounds_scales_with_hex_size_relative_to_the_editor_default() {
        let mut layer = ImageLayer::new();
        layer.size = iced::Size::new(16.0, 32.0);
        layer.position = Point::new(0.0, 0.0);

        // At the editor's own hex size (16.0), bounds pass through unchanged.
        let at_default = layer.bounds(16.0).unwrap();
        assert_eq!(at_default.width, 16.0);
        assert_eq!(at_default.height, 32.0);

        // At double the hex size, the image bounds should double too.
        let doubled = layer.bounds(32.0).unwrap();
        assert_eq!(doubled.width, 32.0);
        assert_eq!(doubled.height, 64.0);
    }

    #[test]
    fn draw_without_an_image_does_nothing() {
        let layer = ImageLayer::new();
        let mut renderer = MockRenderer::default();
        layer.draw(&mut renderer);
        assert!(renderer.images.is_empty());
    }

    #[test]
    fn draw_with_an_image_forwards_bounds_and_opacity() {
        let id = ImageId::from_raw(1);
        let mut layer = ImageLayer::new_with(id);
        layer.size = iced::Size::new(3.0, 4.0);
        layer.position = Point::new(1.0, 2.0);

        layer.set_opacity(0.7);

        let mut renderer = MockRenderer::default();
        layer.draw(&mut renderer);

        assert_eq!(renderer.images.len(), 1);
        let (bounds, drawn_id, opacity) = renderer.images[0];
        assert_eq!(bounds, layer.get_bounds());
        assert_eq!(drawn_id, id);
        assert_eq!(opacity, 0.7);
    }
}
