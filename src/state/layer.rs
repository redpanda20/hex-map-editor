use std::collections::HashSet;

use iced::Color;

use crate::state::HexCoord;

pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub color: Color,

    // Painted tiles
    pub tiles: HashSet<HexCoord>,
}

impl Layer {
    pub fn new(name: impl Into<String>, color: Color) -> Layer {
        let name = name.into();
        let tiles = HashSet::new();
        Self {
            name,
            visible: true,
            tiles,
            color,
        }
    }
}
