pub mod image;
pub mod noise;
pub mod overlay;
pub mod tiles;
pub mod unknown;

use std::fmt::Display;

use iced::Rectangle;

use crate::domain::{
    RenderTarget,
    id::LayerId,
    layer::{
        image::ImageLayer, noise::PerlinNoiseLayer, tiles::SparseTiles, unknown::UnknownLayer,
    },
};

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,

    pub kind: LayerInner,
}

impl Layer {
    pub fn new(name: impl Into<String>, kind: LayerInner) -> Self {
        Self {
            id: LayerId::next(),
            name: name.into(),
            visible: true,
            kind,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerKind {
    #[default]
    Tiles,
    Noise,
    Image,
}
impl Display for LayerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerKind::Tiles => write!(f, "Tiles"),
            LayerKind::Noise => write!(f, "Noise"),
            LayerKind::Image => write!(f, "Image"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LayerInner {
    Tiles(SparseTiles),
    Perlin(PerlinNoiseLayer),
    Image(ImageLayer),
    /// A layer kind this build doesn't recognise - see [`UnknownLayer`].
    Unknown(UnknownLayer),
}

impl LayerInnerImpl for LayerInner {
    fn bounds(&self, hex_size: f32) -> Option<Rectangle> {
        match self {
            LayerInner::Tiles(inner) => inner.bounds(hex_size),
            LayerInner::Perlin(inner) => inner.bounds(hex_size),
            LayerInner::Image(inner) => inner.bounds(hex_size),
            LayerInner::Unknown(inner) => inner.bounds(hex_size),
        }
    }

    fn draw(&self, renderer: &mut dyn RenderTarget) {
        match self {
            LayerInner::Tiles(inner) => inner.draw(renderer),
            LayerInner::Perlin(inner) => inner.draw(renderer),
            LayerInner::Image(inner) => inner.draw(renderer),
            LayerInner::Unknown(inner) => inner.draw(renderer),
        }
    }
}

pub trait LayerInnerImpl: std::fmt::Debug + LayerInnerImplClone {
    fn bounds(&self, hex_size: f32) -> Option<Rectangle>;

    fn draw(&self, renderer: &mut dyn RenderTarget);
}

pub trait LayerInnerImplClone {
    fn clone_box(&self) -> Box<dyn LayerInnerImpl>;
}

impl<T> LayerInnerImplClone for T
where
    T: 'static + LayerInnerImpl + Clone,
{
    fn clone_box(&self) -> Box<dyn LayerInnerImpl> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::layer::tiles::SparseTiles;

    #[test]
    fn layer_kind_display_matches_labels_used_in_the_ui() {
        assert_eq!(LayerKind::Tiles.to_string(), "Tiles");
        assert_eq!(LayerKind::Noise.to_string(), "Noise");
        assert_eq!(LayerKind::Image.to_string(), "Image");
    }

    #[test]
    fn layer_kind_default_is_tiles() {
        assert_eq!(LayerKind::default(), LayerKind::Tiles);
    }

    #[test]
    fn new_layer_is_visible_with_the_given_name() {
        let kind = LayerInner::Tiles(SparseTiles::new(iced::Color::BLACK));
        let layer = Layer::new("My Layer", kind);

        assert_eq!(layer.name, "My Layer");
        assert!(layer.visible);
    }

    #[test]
    fn each_new_layer_gets_a_distinct_id() {
        let a = Layer::new("A", LayerInner::Tiles(SparseTiles::new(iced::Color::BLACK)));
        let b = Layer::new("B", LayerInner::Tiles(SparseTiles::new(iced::Color::BLACK)));
        assert_ne!(a.id, b.id);
    }
}
