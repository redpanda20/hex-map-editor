use std::collections::HashSet;

use iced::{Color, Rectangle};

use crate::domain::{HexBounds, HexCoord};

#[derive(Debug, Clone)]
pub struct SparseTiles {
    pub tiles: HashSet<HexCoord>,
    colour: Color,
}

impl SparseTiles {
    pub fn new(colour: Color) -> Self {
        SparseTiles {
            tiles: HashSet::new(),
            colour,
        }
    }

    pub fn new_with(colour: Color, tiles: HashSet<HexCoord>) -> Self {
        SparseTiles { tiles, colour }
    }

    pub fn get_colour(&self) -> Color {
        self.colour
    }

    pub fn set_colour(&mut self, colour: Color) {
        self.colour = colour
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    // Used for flood fill
    pub fn get_all_tiles(&self) -> &HashSet<HexCoord> {
        &self.tiles
    }
}

impl SparseTiles {
    pub(super) fn exists_at(&self, location: &HexCoord) -> bool {
        self.tiles.contains(location)
    }

    pub(super) fn colour_at(&self, _location: &HexCoord) -> Color {
        self.colour
    }

    pub(super) fn get_bounds(&self) -> Option<HexBounds> {
        HexBounds::from_hexes(self.tiles.clone())
    }

    pub(super) fn get_bounding_box(&self, hex_size: f32) -> Option<Rectangle> {
        let mut iter = self.tiles.iter();
        let first = iter.next()?.to_cartesian() * hex_size;

        let (mut min_x, mut max_x) = (first.x, first.x);
        let (mut min_y, mut max_y) = (first.y, first.y);

        for coord in iter {
            let point = coord.to_cartesian() * hex_size;
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

        Some(Rectangle {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        })
    }
}
