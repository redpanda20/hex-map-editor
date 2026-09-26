use iced::{Point, Rectangle, Size};

use crate::domain::{
    RenderTarget,
    assets::FileKind,
    edit::{SetImageLockAspectRatio, SetImageOpacity, SetImagePosition, SetImageSize},
    id::ImageId,
    inspect::{Property, PropertyInfo},
    layer::{Inspectable, Renderable},
};

/// TODO: Route this appropriately.
/// Shouldn't be known outside of UI but export
/// needs to know for correct sizing of images.
pub const EDITOR_HEX_SIZE: f32 = 16.0;

#[derive(Debug, Default, Clone)]
pub struct ImageLayer {
    pub image: Option<ImageId>,
    size: Size,
    pub lock_aspect_ratio: bool,
    pub position: Point,
    opacity: f32,
}

impl ImageLayer {
    pub fn new() -> Self {
        Self {
            image: None,
            size: Size::default(),
            lock_aspect_ratio: true,
            position: Point::default(),
            opacity: 1.0,
        }
    }

    pub fn new_with(image: ImageId) -> Self {
        Self {
            image: Some(image),
            size: Size::default(),
            lock_aspect_ratio: true,
            position: Point::default(),
            opacity: 1.0,
        }
    }

    pub fn get_size(&self) -> Size {
        self.size
    }

    pub fn set_size(&mut self, size: Size) {
        if !self.lock_aspect_ratio
            || self.size.width <= 0.0
            || self.size.height <= 0.0
            || size.width <= 0.0
        {
            self.size = size;
            return;
        }

        let aspect_ratio = self.size.height / self.size.width;
        self.size = Size::new(size.width, size.width * aspect_ratio);
    }

    pub fn set_size_ignore_aspect_ratio(&mut self, size: Size) {
        self.size = size
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

impl Renderable for ImageLayer {
    fn bounds(&self, hex_size: f32) -> Option<Rectangle> {
        let relative_size = hex_size / EDITOR_HEX_SIZE;
        Some(self.get_bounds() * relative_size)
    }

    fn draw(&self, renderer: &mut dyn RenderTarget) {
        if let Some(image) = self.image {
            renderer.draw_image(self.get_bounds(), image, self.opacity);
        }
    }
}

impl Inspectable for ImageLayer {
    fn properties<'a>(&'a self) -> Vec<Property<'a>> {
        vec![
            // File
            Property::File {
                info: PropertyInfo { label: "File" },
                display_value: None,
                kind: FileKind::Image,
            },
            // Opacity
            Property::BoundedFloat {
                info: PropertyInfo { label: "Opacity" },
                value: self.opacity as f64,
                range: 0.0..=1.0,
                on_submit: Box::new(|value, id| {
                    Box::new(SetImageOpacity {
                        id,
                        opacity: value as f32,
                    })
                }),
            },
            // X, Y
            Property::Group {
                info: Some(PropertyInfo { label: "Position" }),
                children: vec![
                    Property::Float {
                        info: PropertyInfo { label: "X" },
                        value: self.position.x as f64,
                        on_submit: Box::new(|value, id| {
                            let position = Point::new(value as f32, self.position.y);
                            Box::new(SetImagePosition { id, position })
                        }),
                    },
                    Property::Float {
                        info: PropertyInfo { label: "Y" },
                        value: self.position.y as f64,
                        on_submit: Box::new(|value, id| {
                            let position = Point::new(self.position.x, value as f32);
                            Box::new(SetImagePosition { id, position })
                        }),
                    },
                ],
            },
            // Width, Height
            Property::Group {
                info: None,
                children: vec![
                    Property::Float {
                        info: PropertyInfo { label: "Width" },
                        value: self.size.width as f64,
                        on_submit: Box::new(|value, id| {
                            let size = Size::new(value as f32, self.size.height);
                            Box::new(SetImageSize { id, size })
                        }),
                    },
                    Property::Float {
                        info: PropertyInfo { label: "Height" },
                        value: self.size.height as f64,
                        on_submit: Box::new(|value, id| {
                            let size = Size::new(self.size.width, value as f32);
                            Box::new(SetImageSize { id, size })
                        }),
                    },
                ],
            },
            // Lock aspect ratio
            Property::Boolean {
                info: PropertyInfo {
                    label: "Lock aspect ratio",
                },
                value: self.lock_aspect_ratio,
                on_submit: Box::new(|value, id| {
                    Box::new(SetImageLockAspectRatio {
                        id,
                        lock_aspect_ratio: value,
                    })
                }),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use iced::Point;

    use super::*;
    use crate::domain::ports::render::MockRenderer;

    #[test]
    fn new_layer_has_no_image_and_full_opacity() {
        let layer = ImageLayer::new();
        assert_eq!(layer.image, None);
        assert_eq!(layer.get_opacity(), 1.0);
        assert!(layer.lock_aspect_ratio);
    }

    #[test]
    fn new_with_sets_the_image() {
        let id = ImageId::from_raw(1);
        let layer = ImageLayer::new_with(id);
        assert_eq!(layer.image, Some(id));
    }

    #[test]
    fn set_size_updates_both_dimensions_when_aspect_ratio_is_unlocked() {
        let mut layer = ImageLayer::new();
        layer.size = Size::new(100.0, 50.0);
        layer.lock_aspect_ratio = false;

        layer.set_size(Size::new(200.0, 300.0));

        assert_eq!(layer.size, Size::new(200.0, 300.0));
    }

    #[test]
    fn set_size_preserves_aspect_ratio_when_locked() {
        let mut layer = ImageLayer::new();
        layer.size = Size::new(100.0, 50.0);
        layer.lock_aspect_ratio = true;

        layer.set_size(Size::new(200.0, 300.0));

        assert_eq!(layer.size.width, 200.0);
        assert_eq!(layer.size.height, 100.0);
    }

    #[test]
    fn disabling_aspect_ratio_lock_allows_independent_dimensions() {
        let mut layer = ImageLayer::new();
        layer.size = Size::new(100.0, 50.0);
        layer.lock_aspect_ratio = true;
        layer.set_size(Size::new(200.0, 300.0));

        layer.lock_aspect_ratio = false;
        layer.set_size(Size::new(400.0, 300.0));

        assert_eq!(layer.size, Size::new(400.0, 300.0));
    }

    #[test]
    fn set_size_does_not_divide_by_zero_for_zero_sized_layer() {
        let mut layer = ImageLayer::new();
        layer.size = Size::new(0.0, 0.0);
        layer.lock_aspect_ratio = true;

        layer.set_size(Size::new(200.0, 100.0));

        assert_eq!(layer.size, Size::new(200.0, 100.0));
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
