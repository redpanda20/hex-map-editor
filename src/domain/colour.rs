use iced::Color;

/// A colour in HSVA space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsva {
    // Hue (degrees): 0.0 .. 360.0
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub alpha: f32,
}

impl From<Color> for Hsva {
    fn from(colour: Color) -> Self {
        let r = colour.r;
        let g = colour.g;
        let b = colour.b;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta.abs() < f32::EPSILON {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta).rem_euclid(6.0))
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        }
        .rem_euclid(360.0);

        let s = if max.abs() < f32::EPSILON {
            0.0
        } else {
            delta / max
        };
        let v = max;

        Hsva {
            hue: h,
            saturation: s,
            value: v,
            alpha: colour.a,
        }
    }
}

impl From<Hsva> for Color {
    fn from(colour: Hsva) -> Self {
        let saturation = colour.saturation.clamp(0.0, 1.0);
        let value = colour.value.clamp(0.0, 1.0);

        let c = value * saturation;
        let h_prime = colour.hue.rem_euclid(360.0) / 60.0;
        let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
        let m = value - c;

        let (r1, g1, b1) = match h_prime as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color::from_rgba(r1 + m, g1 + m, b1 + m, colour.alpha)
    }
}
