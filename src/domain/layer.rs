pub mod noise;
pub mod tile_store;

use std::fmt::Display;

use iced::Color;
pub use tile_store::SparseTiles;

use crate::domain::HexCoord;

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
    pub fn draw<T>(
        &self,
        target: &mut T,
        coords: impl Iterator<Item = HexCoord>,
        mut draw: impl FnMut(&mut T, HexCoord, Color),
    ) {
        for coord in coords {
            match self {
                LayerInner::Tiles(sparse_tiles) => {
                    if sparse_tiles.exists_at(&coord) {
                        draw(target, coord, sparse_tiles.colour_at(&coord))
                    }
                }
                LayerInner::InvertedTiles(sparse_tiles) => {
                    if !sparse_tiles.exists_at(&coord) {
                        draw(target, coord, sparse_tiles.colour_at(&coord))
                    }
                }
                LayerInner::Perlin(perlin_noise_layer) => {
                    if perlin_noise_layer.exists_at(&coord) {
                        draw(target, coord, perlin_noise_layer.colour_at(&coord))
                    }
                }
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
