pub mod noise;
pub mod tile_store;

use iced::Color;
pub use tile_store::SparseTiles;

use crate::domain::HexCoord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerType {
    Tiles,
    PerlinNoise,
}
impl Default for LayerType {
    fn default() -> Self {
        LayerType::Tiles
    }
}
impl ToString for LayerType {
    fn to_string(&self) -> String {
        match self {
            LayerType::Tiles => "Tile".to_string(),
            LayerType::PerlinNoise => "Noise".to_string(),
        }
    }
}

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

pub struct Layer {
    pub name: String,
    pub visible: bool,

    pub inner: LayerInner,
}

