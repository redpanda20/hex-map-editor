pub mod image;
pub mod noise;
pub mod overlay;
pub mod tiles;

use std::fmt::Display;

use iced::Rectangle;

use crate::domain::{
    RenderTarget,
    id::LayerId,
    layer::{image::ImageLayer, noise::PerlinNoiseLayer, tiles::SparseTiles},
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
}

impl LayerInnerImpl for LayerInner {
    fn bounds(&self, hex_size: f32) -> Option<Rectangle> {
        match self {
            LayerInner::Tiles(inner) => inner.bounds(hex_size),
            LayerInner::Perlin(inner) => inner.bounds(hex_size),
            LayerInner::Image(inner) => inner.bounds(hex_size),
        }
    }

    fn draw(&self, renderer: &mut dyn RenderTarget) {
        match self {
            LayerInner::Tiles(inner) => inner.draw(renderer),
            LayerInner::Perlin(inner) => inner.draw(renderer),
            LayerInner::Image(inner) => inner.draw(renderer),
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
