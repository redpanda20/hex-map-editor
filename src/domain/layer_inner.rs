mod grid_overlay;
mod noise;
mod tile_store;

use std::fmt::Display;

pub use grid_overlay::HexGridOverlay;
use iced::Rectangle;
pub use noise::{NoiseOctaves, PerlinNoiseLayer};
pub use tile_store::SparseTiles;

use crate::domain::render::RenderTarget;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    #[default]
    Tiles,
    PerlinNoise,
    Utility,
}

impl Display for LayerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerKind::Tiles => write!(f, "Tile"),
            LayerKind::PerlinNoise => write!(f, "Noise"),
            LayerKind::Utility => write!(f, "Utility"),
        }
    }
}

#[allow(
    clippy::large_enum_variant,
    reason = "Indirection via Boxed<Trait> will be implemented in the future"
)]
#[derive(Debug, Clone)]
pub enum LayerInner {
    Tiles(SparseTiles),
    Perlin(PerlinNoiseLayer),
}

pub trait LayerInnerImpl {
    fn kind(&self) -> LayerKind;

    fn bounds(&self, hex_size: f32) -> Option<Rectangle>;

    fn draw(&self, renderer: &mut dyn RenderTarget);
}

impl LayerInnerImpl for LayerInner {
    fn kind(&self) -> LayerKind {
        match self {
            LayerInner::Tiles(sparse_tiles) => sparse_tiles.kind(),
            LayerInner::Perlin(perlin_noise_layer) => perlin_noise_layer.kind(),
        }
    }

    fn bounds(&self, hex_size: f32) -> Option<Rectangle> {
        match self {
            LayerInner::Tiles(sparse_tiles) => sparse_tiles.bounds(hex_size),
            LayerInner::Perlin(perlin_noise_layer) => perlin_noise_layer.bounds(hex_size),
        }
    }

    fn draw(&self, renderer: &mut dyn RenderTarget) {
        match self {
            LayerInner::Tiles(sparse_tiles) => sparse_tiles.draw(renderer),
            LayerInner::Perlin(perlin_noise_layer) => perlin_noise_layer.draw(renderer),
        }
    }
}
