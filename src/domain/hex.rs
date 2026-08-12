use std::collections::HashSet;

use iced::{Rectangle, Vector};
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
        let y = (3.0_f32.sqrt() * 0.5 * q) + (3.0_f32.sqrt() * r);

        Vector { x, y }
    }

    pub fn from_cartesian(vec: Vector) -> HexCoord {
        let q = 2.0 / 3.0 * vec.x;
        let r = -1.0 / 3.0 * vec.x + (3.0_f32.sqrt() / 3.0) * vec.y;

        // Axial to cubic coordinates
        let cube_x = q;
        let cube_z = r;
        let cube_y = -cube_x - cube_z;

        frac_round(cube_x, cube_z, cube_y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HexBounds {
    pub col_min: i32,
    pub col_max: i32,
    pub row_min: i32,
    pub row_max: i32,
}

impl HexBounds {
    pub fn contains(&self, coord: HexCoord) -> bool {
        coord.col >= self.col_min
            && coord.col <= self.col_max
            && coord.row >= self.row_min
            && coord.row <= self.row_max
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
}

pub fn hexes_in_range(
    col_min: i32,
    col_max: i32,
    row_min: i32,
    row_max: i32,
) -> impl Iterator<Item = HexCoord> {
    (col_min..=col_max).flat_map(move |col| {
        (row_min..=row_max).map(move |row| HexCoord {
            col,
            row: row - col / 2,
        })
    })
}

pub fn rect_to_range(rect: Rectangle, hex_size: f32) -> (i32, i32, i32, i32) {
    let inv_hex_w = 1.0 / (hex_size * 1.5);
    let inv_hex_h = 1.0 / (hex_size * 3.0_f32.sqrt());

    let col_min = (rect.x * inv_hex_w).floor() as i32;
    let col_max = col_min + (rect.width * inv_hex_w).ceil() as i32;

    let row_min = (rect.y * inv_hex_h).floor() as i32;
    let row_max = row_min + (rect.height * inv_hex_h).ceil() as i32;

    (col_min, col_max, row_min, row_max)
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
pub fn flood_fill(
    start: HexCoord,
    tiles: &HashSet<HexCoord>,
    bounds: HexBounds,
) -> Option<HashSet<HexCoord>> {
    let target_state = tiles.contains(&start);

    let mut visited = HashSet::new();
    let mut stack = vec![start];

    while let Some(coord) = stack.pop() {
        if visited.contains(&coord) {
            continue;
        }
        if !bounds.contains(coord) || visited.len() > FLOOD_FILL_TILE_CAP {
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
