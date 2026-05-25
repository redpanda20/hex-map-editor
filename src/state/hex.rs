// Coordinate system: offset coordinates (even-q flat-top)

use serde::{Deserialize, Serialize};

// Scale is different in export
pub const HEX_SIZE: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HexCoord {
    pub col: i32,
    pub row: i32,
}

impl HexCoord {
    pub fn new(col: i32, row: i32) -> Self {
        Self { col, row }
    }

    /// Convert hex offset coord → pixel centre (flat-top hexagons).
    /// `size` is the circumradius (centre → vertex).
    pub fn to_pixel(self, hex_size: f32) -> (f32, f32) {
        let w = hex_size * 2.0;
        let h = hex_size * (3.0_f32).sqrt();
        let x = self.col as f32 * w * 0.75;
        let y = self.row as f32 * h + if self.col % 2 != 0 { h * 0.5 } else { 0.0 };
        (x, y)
    }

    /// Snap a pixel position to the nearest hex coordinate.
    pub fn from_pixel(px: f32, py: f32, hex_size: f32) -> Self {
        // Use cube-coordinate rounding for accuracy.
        let w = hex_size * 2.0;
        let h = hex_size * (3.0_f32).sqrt();

        let col_approx = px / (w * 0.75);
        let col = col_approx.round() as i32;
        let offset = if col % 2 != 0 { h * 0.5 } else { 0.0 };
        let row = ((py - offset) / h).round() as i32;

        // Refine: check the candidate and its neighbors.
        let candidates = [
            HexCoord::new(col, row),
            HexCoord::new(col - 1, row),
            HexCoord::new(col + 1, row),
            HexCoord::new(col - 1, row - 1),
            HexCoord::new(col + 1, row - 1),
            HexCoord::new(col - 1, row + 1),
            HexCoord::new(col + 1, row + 1),
        ];

        candidates
            .iter()
            .min_by(|a, b| {
                let (ax, ay) = a.to_pixel(hex_size);
                let (bx, by) = b.to_pixel(hex_size);
                let da = (ax - px).powi(2) + (ay - py).powi(2);
                let db = (bx - px).powi(2) + (by - py).powi(2);
                da.partial_cmp(&db).unwrap()
            })
            .copied()
            .unwrap()
    }
}
