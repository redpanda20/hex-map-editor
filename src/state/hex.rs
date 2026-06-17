use iced::Vector;
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
