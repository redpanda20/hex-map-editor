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

#[cfg(test)]
mod tests {
    use iced::Color;

    use super::*;
    use crate::domain::render::MockRenderer;

    fn coord(col: i32, row: i32) -> HexCoord {
        HexCoord { col, row }
    }

    #[test]
    fn paint_reports_whether_it_changed_anything() {
        let mut tiles = SparseTiles::new(Color::BLACK);
        assert!(tiles.paint(coord(0, 0)));
        assert!(!tiles.paint(coord(0, 0)), "painting twice is a no-op");
    }

    #[test]
    fn erase_reports_whether_it_changed_anything() {
        let mut tiles = SparseTiles::new(Color::BLACK);
        assert!(!tiles.erase(coord(0, 0)), "nothing to erase yet");
        tiles.paint(coord(0, 0));
        assert!(tiles.erase(coord(0, 0)));
    }

    #[test]
    fn paint_multiple_returns_only_newly_painted_coords() {
        let mut tiles = SparseTiles::new(Color::BLACK);
        tiles.paint(coord(0, 0));

        let changed = tiles.paint_multiple([coord(0, 0), coord(1, 0), coord(2, 0)]);
        assert_eq!(changed, vec![coord(1, 0), coord(2, 0)]);
    }

    #[test]
    fn erase_multiple_returns_only_actually_erased_coords() {
        let mut tiles = SparseTiles::new(Color::BLACK);
        tiles.paint_multiple([coord(0, 0), coord(1, 0)]);

        let changed = tiles.erase_multiple([coord(0, 0), coord(5, 5)]);
        assert_eq!(changed, vec![coord(0, 0)]);
    }

    #[test]
    fn invert_toggles_the_inverted_flag() {
        let mut tiles = SparseTiles::new(Color::BLACK);
        assert!(!tiles.is_inverted());
        tiles.invert();
        assert!(tiles.is_inverted());
        tiles.invert();
        assert!(!tiles.is_inverted());
    }

    #[test]
    fn is_empty_reflects_the_tile_set() {
        let mut tiles = SparseTiles::new(Color::BLACK);
        assert!(tiles.is_empty());
        tiles.paint(coord(0, 0));
        assert!(!tiles.is_empty());
    }

    #[test]
    fn bounds_is_none_for_an_empty_layer() {
        let tiles = SparseTiles::new(Color::BLACK);
        assert!(tiles.bounds(16.0).is_none());
    }

    /// The exact set of hexes `draw()` will iterate for a given viewport,
    /// computed the same way `draw()` does internally
    /// (`HexBounds::from_rect(renderer.get_bounds())`). Used instead of a
    /// hand-picked hex range so these tests don't have to duplicate (and
    /// risk getting wrong) `HexBounds::from_rect`'s corner-rounding
    /// behaviour.
    fn visible_hexes(bounds: Rectangle) -> Vec<HexCoord> {
        HexBounds::from_rect(bounds).into_hexes().collect()
    }

    #[test]
    fn draw_fills_only_painted_hexes_within_the_viewport() {
        let mut tiles = SparseTiles::new(Color::BLACK);
        tiles.paint(coord(0, 0));

        let bounds = HexBounds::new(-1, 1, -1, 1).into_rect();
        let hexes = visible_hexes(bounds);
        assert!(
            hexes.contains(&coord(0, 0)),
            "sanity: origin should be in view"
        );

        let mut renderer = MockRenderer::with_bounds(bounds);
        tiles.draw(&mut renderer);

        assert_eq!(
            renderer.fills.len(),
            1,
            "only the one painted hex should be filled"
        );
        assert_eq!(renderer.fills[0].1, Color::BLACK);
    }

    #[test]
    fn draw_with_inverted_flag_fills_the_unpainted_hexes_instead() {
        let mut tiles = SparseTiles::new(Color::BLACK);
        tiles.paint(coord(0, 0));
        tiles.invert();

        let bounds = HexBounds::new(-1, 1, -1, 1).into_rect();
        let hexes = visible_hexes(bounds);
        assert!(
            hexes.contains(&coord(0, 0)),
            "sanity: origin should be in view"
        );

        let mut renderer = MockRenderer::with_bounds(bounds);
        tiles.draw(&mut renderer);

        // Every visible hex except the one painted one should now be filled.
        assert_eq!(renderer.fills.len(), hexes.len() - 1);
    }
}
