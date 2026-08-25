use iced::Rectangle;

use crate::domain::{RenderTarget, id::ImageId, layer::LayerInnerImpl};

#[derive(Debug, Default, Clone)]
pub struct ImageLayer {
    pub image: Option<ImageId>,
    pub bounds: Rectangle,
    opacity: f32,
}

impl ImageLayer {
    pub fn new() -> Self {
        Self {
            image: None,
            bounds: Rectangle::default(),
            opacity: 1.0,
        }
    }

    pub fn new_with(image: ImageId) -> Self {
        Self {
            image: Some(image),
            bounds: Rectangle::default(),
            opacity: 1.0,
        }
    }

    pub fn get_opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }
}

impl LayerInnerImpl for ImageLayer {
    fn bounds(&self, _hex_size: f32) -> Option<Rectangle> {
        Some(self.bounds)
    }

    fn draw(&self, renderer: &mut dyn RenderTarget) {
        if let Some(image) = self.image {
            renderer.draw_image(self.bounds, image, self.opacity);
        }
    }
}
