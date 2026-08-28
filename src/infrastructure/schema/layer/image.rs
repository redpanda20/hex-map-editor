use iced::Rectangle;
use serde::{Deserialize, Serialize};

use crate::domain::{id::ImageId, layer::image::ImageLayer};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RectangleV1 {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<Rectangle> for RectangleV1 {
    fn from(rect: Rectangle) -> Self {
        RectangleV1 {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl From<RectangleV1> for Rectangle {
    fn from(rect: RectangleV1) -> Self {
        Rectangle {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageLayerV1 {
    /// References an entry in `resources.json` by id,
    /// (see `schema::resource`).
    ///
    /// A given resource id may be referenced by more than one
    pub resource: Option<u64>,
    pub bounds: RectangleV1,
    pub opacity: f32,
}

impl From<&ImageLayer> for ImageLayerV1 {
    fn from(layer: &ImageLayer) -> Self {
        ImageLayerV1 {
            resource: layer.image.map(ImageId::raw),
            bounds: layer.bounds.into(),
            opacity: layer.get_opacity(),
        }
    }
}

impl ImageLayerV1 {
    /// `resolve` maps a persisted resource id to the `ImageId` it was
    /// registered under in this session's `AssetStore` - see
    /// `schema::Document::into_scene`, which registers each resource once
    /// up front and shares the id across every layer that references it.
    pub fn into_domain(self, resolve: impl Fn(u64) -> Option<ImageId>) -> ImageLayer {
        let mut layer = match self.resource.and_then(resolve) {
            Some(id) => ImageLayer::new_with(id),
            None => ImageLayer::new(),
        };
        layer.bounds = self.bounds.into();
        layer.set_opacity(self.opacity);
        layer
    }
}
