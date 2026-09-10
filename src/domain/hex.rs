use std::collections::HashSet;

use iced::{Point, Rectangle, Size, Vector};

/// Taken from Rust nightly:
/// https://doc.rust-lang.org/std/f32/consts/constant.SQRT_3.html
const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;

/// Flat topped axial grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub col: i32,
    pub row: i32,
}

impl HexCoord {
    const NEIGHBOR_OFFSETS: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

    pub fn neighbors(self) -> [HexCoord; 6] {
        Self::NEIGHBOR_OFFSETS.map(|(dc, dr)| HexCoord {
            col: self.col + dc,
            row: self.row + dr,
        })
    }

    pub fn to_cartesian(self) -> Vector {
        let q = self.col as f32;
        let r = self.row as f32;

        let x = 1.5 * q;
        let y = (SQRT_3 * 0.5 * q) + (SQRT_3 * r);

        Vector { x, y }
    }

    pub fn from_cartesian(vec: Vector) -> HexCoord {
        let q = 2.0 / 3.0 * vec.x;
        let r = -1.0 / 3.0 * vec.x + (SQRT_3 / 3.0) * vec.y;

        // Axial to cubic coordinates
        let cube_x = q;
        let cube_z = r;
        let cube_y = -cube_x - cube_z;

        frac_round(cube_x, cube_z, cube_y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HexBounds {
    col_min: i32,
    col_max: i32,
    row_min: i32,
    row_max: i32,
}

impl HexBounds {
    pub fn contains(&self, coord: HexCoord) -> bool {
        coord.col >= self.col_min
            && coord.col <= self.col_max
            && coord.row >= self.row_min
            && coord.row <= self.row_max
    }

    pub fn union(&self, other: &Self) -> Self {
        let col_min = self.col_min.min(other.col_min);
        let col_max = self.col_max.max(other.col_max);
        let row_min = self.row_min.min(other.row_min);
        let row_max = self.row_max.max(other.row_max);

        HexBounds {
            col_min,
            col_max,
            row_min,
            row_max,
        }
    }
}

impl HexBounds {
    /// Create bounds from explicit values
    pub fn new(col_min: i32, col_max: i32, row_min: i32, row_max: i32) -> Self {
        debug_assert!(col_min <= col_max);
        debug_assert!(row_min <= row_max);
        Self {
            col_min,
            col_max,
            row_min,
            row_max,
        }
    }

    /// Expand bounds by `padding`.
    /// Negative values will shrink the bounds
    pub fn expand(self, padding: i32) -> Self {
        Self {
            col_min: self.col_min - padding,
            col_max: self.col_max + padding,
            row_min: self.row_min - padding,
            row_max: self.row_max + padding,
        }
    }

    /// Move the bounds on the x-axis by `offset`
    pub fn translate_x(self, offset: i32) -> Self {
        Self {
            col_min: self.col_min + offset,
            col_max: self.col_max + offset,
            row_min: self.row_min,
            row_max: self.row_max,
        }
    }

    /// Create an axial bounding box, that conservatively covers every hex in `rect`.
    ///
    /// Excludes intersecting hexes with centres outside of `rect`.
    pub fn from_rect(rect: Rectangle) -> Self {
        let coords = [
            Vector::new(rect.x, rect.y),
            Vector::new(rect.x + rect.width, rect.y),
            Vector::new(rect.x, rect.y + rect.height),
            Vector::new(rect.x + rect.width, rect.y + rect.height),
        ]
        .map(HexCoord::from_cartesian);

        Self::from_hexes(coords).expect("A rectangle always produces four corners")
    }

    pub fn into_rect(&self) -> Rectangle {
        let min = HexCoord {
            col: self.col_min,
            row: self.row_min,
        }
        .to_cartesian();
        let max = HexCoord {
            col: self.col_max,
            row: self.row_max,
        }
        .to_cartesian();

        let top_left = Point { x: min.x, y: min.y };
        let size = Size {
            width: max.x - min.x,
            height: max.y - min.y,
        };

        Rectangle::new(top_left, size)
    }

    pub fn from_hexes(source: impl IntoIterator<Item = HexCoord>) -> Option<HexBounds> {
        let mut iter = source.into_iter();
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

    pub fn into_hexes(self) -> impl Iterator<Item = HexCoord> {
        (self.col_min..=self.col_max).flat_map(move |col| {
            (self.row_min..=self.row_max).map(move |row| HexCoord { col, row })
        })
    }
}

fn frac_round(frac_q: f32, frac_r: f32, frac_s: f32) -> HexCoord {
    let mut q = frac_q.round();
    let mut r = frac_r.round();
    let s = frac_s.round();

    let q_diff = (q - frac_q).abs();
    let r_diff = (r - frac_r).abs();
    let s_diff = (s - frac_s).abs();

    if q_diff > r_diff && q_diff > s_diff {
        q = -r - s
    } else if r_diff > s_diff {
        r = -q - s
    }

    let col = q as i32;
    let row = r as i32;

    HexCoord { col, row }
}

/// Cap in case user has created a map too large to performantly process
const FLOOD_FILL_TILE_CAP: usize = 5_000;

/// Flood-fills the connected region of hexes sharing `start`'s painted state (painted or empty), constrained to `bounds`.
///
/// Returns `None` if the region is effectively unbounded.
/// This is the case when the fill reaches the bounds or exceeds the tile cap
pub fn flood_fill(start: HexCoord, tiles: &HashSet<HexCoord>) -> Option<HashSet<HexCoord>> {
    let target_state = tiles.contains(&start);
    let bounds = HexBounds::from_hexes(tiles.clone())?;

    let mut visited = HashSet::new();
    let mut stack = vec![start];

    while let Some(coord) = stack.pop() {
        if visited.contains(&coord) {
            continue;
        }
        if !bounds.contains(coord) || visited.len() >= FLOOD_FILL_TILE_CAP {
            return None;
        }
        visited.insert(coord);

        for neighbor in coord.neighbors() {
            if !visited.contains(&neighbor) && tiles.contains(&neighbor) == target_state {
                stack.push(neighbor);
            }
        }
    }

    Some(visited)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(col: i32, row: i32) -> HexCoord {
        HexCoord { col, row }
    }

    #[test]
    fn neighbors_use_the_documented_offsets() {
        let got = c(0, 0).neighbors();
        let expected = [c(1, 0), c(1, -1), c(0, -1), c(-1, 0), c(-1, 1), c(0, 1)];
        assert_eq!(got, expected);

        // Neighbors are relative, not absolute - check translation holds too.
        let got = c(5, -3).neighbors();
        let expected = [c(6, -3), c(6, -4), c(5, -4), c(4, -3), c(4, -2), c(5, -2)];
        assert_eq!(got, expected);
    }

    #[test]
    fn cartesian_round_trip_is_exact_for_integer_coords() {
        for coord in [
            c(0, 0),
            c(1, 0),
            c(0, 1),
            c(-1, 2),
            c(3, -2),
            c(5, 5),
            c(-4, -4),
            c(10, -7),
        ] {
            let round_tripped = HexCoord::from_cartesian(coord.to_cartesian());
            assert_eq!(
                round_tripped, coord,
                "round-trip failed for {coord:?} (got {round_tripped:?})"
            );
        }
    }

    #[test]
    fn from_cartesian_snaps_to_nearest_hex() {
        // The centre of a hex should always map back to itself, and a point
        // nudged slightly off-centre should still snap to that same hex.
        let centre = c(2, -1).to_cartesian();
        let nudged = iced::Vector::new(centre.x + 0.05, centre.y - 0.05);
        assert_eq!(HexCoord::from_cartesian(nudged), c(2, -1));
    }

    #[test]
    fn bounds_contains_is_inclusive_of_edges() {
        let bounds = HexBounds::new(0, 2, 0, 2);
        assert!(bounds.contains(c(0, 0)));
        assert!(bounds.contains(c(2, 2)));
        assert!(bounds.contains(c(1, 1)));
        assert!(!bounds.contains(c(3, 1)));
        assert!(!bounds.contains(c(1, -1)));
    }

    #[test]
    fn bounds_union_covers_both_inputs() {
        let a = HexBounds::new(0, 2, 0, 2);
        let b = HexBounds::new(5, 7, -3, 1);
        let union = a.union(&b);

        assert!(union.contains(c(0, 0)));
        assert!(union.contains(c(7, -3)));
    }

    #[test]
    fn bounds_expand_grows_symmetrically_and_shrink_with_negative() {
        let bounds = HexBounds::new(0, 2, 0, 2);
        let grown = bounds.expand(1);
        assert!(grown.contains(c(-1, -1)));
        assert!(grown.contains(c(3, 3)));

        let shrunk = bounds.expand(-1);
        assert!(!shrunk.contains(c(0, 1)));
        assert!(shrunk.contains(c(1, 1)));
    }

    #[test]
    fn bounds_translate_x_only_moves_columns() {
        let bounds = HexBounds::new(0, 2, 0, 2);
        let moved = bounds.translate_x(3);
        assert!(moved.contains(c(3, 0)));
        assert!(moved.contains(c(5, 2)));
        assert!(!moved.contains(c(0, 0)));
    }

    #[test]
    fn from_hexes_computes_the_bounding_box() {
        let bounds = HexBounds::from_hexes([c(0, 0), c(2, 3), c(-1, 5)]).unwrap();
        assert!(bounds.contains(c(-1, 0)));
        assert!(bounds.contains(c(2, 5)));
        assert!(!bounds.contains(c(-2, 0)));
        assert!(!bounds.contains(c(3, 0)));
    }

    #[test]
    fn from_hexes_of_empty_iterator_is_none() {
        assert!(HexBounds::from_hexes(std::iter::empty()).is_none());
    }

    #[test]
    fn into_hexes_enumerates_every_cell_in_the_box() {
        let hexes: Vec<_> = HexBounds::new(0, 1, 0, 1).into_hexes().collect();
        assert_eq!(hexes, vec![c(0, 0), c(0, 1), c(1, 0), c(1, 1)]);
    }

    #[test]
    fn flood_fill_of_empty_tile_set_is_none() {
        let tiles: HashSet<HexCoord> = HashSet::new();
        assert!(flood_fill(c(0, 0), &tiles).is_none());
    }

    #[test]
    fn flood_fill_stops_at_a_bounded_hole() {
        // A ring of painted tiles around an empty centre: filling from the
        // centre should only ever touch the centre itself, since every
        // neighbor is painted (a different "state") and the flood never
        // needs to leave the ring's own bounding box.
        let ring: HashSet<HexCoord> = c(0, 0).neighbors().into_iter().collect();

        let filled = flood_fill(c(0, 0), &ring).expect("hole is bounded by the ring");
        assert_eq!(filled, HashSet::from([c(0, 0)]));
    }

    #[test]
    fn flood_fill_returns_none_when_it_escapes_the_bounding_box() {
        // The painted set is a single, distant tile - its bounding box is
        // just that one cell, so starting anywhere else immediately falls
        // outside the bounds and the fill is reported as unbounded.
        let tiles: HashSet<HexCoord> = HashSet::from([c(0, 0)]);
        assert!(flood_fill(c(5, 5), &tiles).is_none());
    }

    #[test]
    fn flood_fill_of_a_fully_painted_region_covers_it_exactly() {
        // Filling from an already-painted tile should walk the whole
        // painted region (its own bounding box) and nothing more.
        let tiles: HashSet<HexCoord> = HashSet::from([c(0, 0), c(1, 0), c(1, -1)]);
        let filled = flood_fill(c(0, 0), &tiles).expect("painted region is its own bounds");
        assert_eq!(filled, tiles);
    }
}
