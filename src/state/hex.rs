use iced::{Rectangle, Vector};
/// Flat topped axial grid

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub col: i32,
    pub row: i32,
}

impl HexCoord {
    pub fn to_pixel(self, hex_size: f32) -> Vector {
        let q = self.col as f32;
        let r = self.row as f32;

        let x = hex_size * (1.5 * q);
        let y = hex_size * ((3.0_f32.sqrt() * 0.5 * q) + (3.0_f32.sqrt() * r));

        Vector { x, y }
    }

    pub fn from_pixel(x: f32, y: f32, hex_size: f32) -> HexCoord {
        let q = (2.0 / 3.0 * x) / hex_size;
        let r = (-1.0 / 3.0 * x + (3.0_f32.sqrt() / 3.0) * y) / hex_size;

        // Axial to cubic coordinates
        let cube_x = q;
        let cube_z = r;
        let cube_y = -cube_x - cube_z;

        frac_round(cube_x, cube_z, cube_y)
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
