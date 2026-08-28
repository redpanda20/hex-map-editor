use std::collections::HashSet;

use iced::Color;
use serde::{Deserialize, Serialize};

use crate::domain::{HexCoord, layer::tiles::SparseTiles};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HexCoordV1 {
    pub col: i32,
    pub row: i32,
}

impl From<HexCoord> for HexCoordV1 {
    fn from(coord: HexCoord) -> Self {
        HexCoordV1 {
            col: coord.col,
            row: coord.row,
        }
    }
}

impl From<HexCoordV1> for HexCoord {
    fn from(coord: HexCoordV1) -> Self {
        HexCoord {
            col: coord.col,
            row: coord.row,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColourV1 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Color> for ColourV1 {
    fn from(colour: Color) -> Self {
        ColourV1 {
            r: colour.r,
            g: colour.g,
            b: colour.b,
            a: colour.a,
        }
    }
}

impl From<ColourV1> for Color {
    fn from(colour: ColourV1) -> Self {
        Color {
            r: colour.r,
            g: colour.g,
            b: colour.b,
            a: colour.a,
        }
    }
}

/// "inverted" is stored as an alternate layer variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesV1 {
    pub colour: ColourV1,
    pub tiles: Vec<HexCoordV1>,
}

impl From<&SparseTiles> for TilesV1 {
    fn from(store: &SparseTiles) -> Self {
        TilesV1 {
            colour: store.colour.into(),
            tiles: store
                .get_all_tiles()
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
        }
    }
}

impl TilesV1 {
    pub fn into_domain(self, inverted: bool) -> SparseTiles {
        let tiles: HashSet<HexCoord> = self.tiles.into_iter().map(Into::into).collect();
        let mut store = SparseTiles::new_with(self.colour.into(), tiles);
        if inverted {
            store.invert();
        }
        store
    }
}
