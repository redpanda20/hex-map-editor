use std::collections::HashSet;

use iced::{Color, Rectangle};

use crate::domain::{HexBounds, HexCoord, RenderTarget};

use super::LayerInnerImpl;

#[derive(Debug, Clone)]
pub struct SparseTiles {
    pub tiles: HashSet<HexCoord>,
    pub colour: Color,
    inverted: bool,
}

impl SparseTiles {
    pub fn new(colour: Color) -> Self {
        SparseTiles {
            tiles: HashSet::new(),
            colour,
            inverted: false,
        }
    }

    pub fn new_with(colour: Color, tiles: HashSet<HexCoord>) -> Self {
        SparseTiles {
            tiles,
            colour,
            inverted: false,
        }
    }

    pub fn paint(&mut self, coord: HexCoord) -> bool {
        self.tiles.insert(coord)
    }

    pub fn erase(&mut self, coord: HexCoord) -> bool {
        self.tiles.remove(&coord)
    }

    /// Paints many coords.
    /// Returns all modified tiles
    pub fn paint_multiple(&mut self, other: impl IntoIterator<Item = HexCoord>) -> Vec<HexCoord> {
        other
            .into_iter()
            .filter(|coord| self.tiles.insert(*coord))
            .collect()
    }

    /// Erases many coords.
    /// Returns all modified tiles
    pub fn erase_multiple(&mut self, other: impl IntoIterator<Item = HexCoord>) -> Vec<HexCoord> {
        other
            .into_iter()
            .filter(|coord| self.tiles.remove(coord))
            .collect()
    }

    pub fn is_inverted(&self) -> bool {
        self.inverted
    }

    pub fn invert(&mut self) {
        self.inverted = !self.inverted;
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    // Used for convert
    pub fn get_all_tiles(&self) -> &HashSet<HexCoord> {
        &self.tiles
    }

    fn get_bounding_box(&self, hex_size: f32) -> Option<Rectangle> {
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

impl LayerInnerImpl for SparseTiles {
    fn bounds(&self, hex_size: f32) -> Option<Rectangle> {
        self.get_bounding_box(hex_size)
    }

    fn draw(&self, renderer: &mut dyn RenderTarget) {
        let bounds = renderer.get_bounds();
        let hexes = HexBounds::from_rect(bounds).into_hexes();

        for coord in hexes {
            if self.tiles.contains(&coord) ^ self.inverted {
                let point = renderer.hex_to_point(&coord);

                renderer.fill_polygon(&point, self.colour);
            }
        }
    }
}
