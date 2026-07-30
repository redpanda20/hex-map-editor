use std::collections::HashSet;

use iced::Color;

use crate::state::HexCoord;

#[derive(Debug, Clone)]
pub struct SparseTiles {
    pub tiles: HashSet<HexCoord>,
    pub colour: Color,
}

impl SparseTiles {
    pub fn new(colour: Color) -> Self {
        SparseTiles {
            tiles: HashSet::new(),
            colour,
        }
    }

    pub fn paint(&mut self, coord: HexCoord) {
        self.tiles.insert(coord);
    }

    pub fn erase(&mut self, coord: HexCoord) {
        self.tiles.remove(&coord);
    }
}
