use std::collections::HashSet;

use iced::{Color, Rectangle};

use crate::state::{HexBounds, HexCoord};

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

    pub fn bounding_box(&self, hex_size: f32) -> Option<Rectangle> {
        let mut iter = self.tiles.iter();
        let first = iter.next()?.to_pixel(hex_size);

        let (mut min_x, mut max_x) = (first.x, first.x);
        let (mut min_y, mut max_y) = (first.y, first.y);

        for coord in iter {
            let point = coord.to_pixel(hex_size);
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

    pub fn hex_bounds(&self) -> Option<HexBounds> {
        let mut iter = self.tiles.iter();
        let first = iter.next()?;

        let (mut col_min, mut col_max) = (first.col, first.col);
        let (mut row_min, mut row_max) = (first.row, first.row);

        for coord in iter {
            col_min = col_min.min(coord.col);
            col_max = col_max.max(coord.col);

            row_min = row_min.min(coord.row);
            row_max = row_max.max(coord.row);
        }

        Some(HexBounds {
            col_min,
            col_max,
            row_min,
            row_max,
        })
    }
}
