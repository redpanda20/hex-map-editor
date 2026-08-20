mod noise;
mod tile_store;

pub use noise::{NoiseOctaves, PerlinNoiseLayer};
pub use tile_store::SparseTiles;

use iced::{Color, Rectangle};
use std::fmt::Display;

use crate::domain::{HexBounds, HexCoord};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LayerType {
    #[default]
    Tiles,
    PerlinNoise,
}

impl Display for LayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerType::Tiles => write!(f, "Tile"),
            LayerType::PerlinNoise => write!(f, "Noise"),
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
    InvertedTiles(SparseTiles),
    Perlin(noise::PerlinNoiseLayer),
}

impl LayerInner {
    pub fn exists_at(&self, location: &HexCoord) -> bool {
        match self {
            LayerInner::Tiles(tiles) | LayerInner::InvertedTiles(tiles) => {
                tiles.exists_at(location)
            }
            LayerInner::Perlin(noise) => noise.exists_at(location),
        }
    }

    pub fn colour_at(&self, location: &HexCoord) -> Color {
        match self {
            LayerInner::Tiles(tiles) | LayerInner::InvertedTiles(tiles) => {
                tiles.colour_at(location)
            }
            LayerInner::Perlin(noise) => noise.colour_at(location),
        }
    }

    pub fn get_bounds(&self) -> Option<HexBounds> {
        match self {
            LayerInner::Tiles(tiles) | LayerInner::InvertedTiles(tiles) => tiles.get_bounds(),
            LayerInner::Perlin(noise) => noise.get_bounds(),
        }
    }

    pub fn get_bounding_box(&self, hex_size: f32) -> Option<Rectangle> {
        match self {
            LayerInner::Tiles(tiles) | LayerInner::InvertedTiles(tiles) => {
                tiles.get_bounding_box(hex_size)
            }
            LayerInner::Perlin(noise) => noise.get_bounding_box(),
        }
    }
}

impl LayerInner {
    pub fn draw<T>(
        &self,
        target: &mut T,
        coords: impl Iterator<Item = HexCoord>,
        mut draw: impl FnMut(&mut T, HexCoord, Color),
    ) {
        for coord in coords {
            if self.exists_at(&coord) {
                draw(target, coord, self.colour_at(&coord))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub visible: bool,

    pub inner: LayerInner,
}
