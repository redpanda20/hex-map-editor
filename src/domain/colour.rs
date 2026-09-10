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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    fn assert_colour_approx(a: Color, b: Color) {
        assert!(
            approx(a.r, b.r) && approx(a.g, b.g) && approx(a.b, b.b) && approx(a.a, b.a),
            "expected {b:?}, got {a:?}"
        );
    }

    #[test]
    fn red_converts_to_hue_zero_full_saturation_and_value() {
        let hsva: Hsva = Color::from_rgb(1.0, 0.0, 0.0).into();
        assert!(approx(hsva.hue, 0.0));
        assert!(approx(hsva.saturation, 1.0));
        assert!(approx(hsva.value, 1.0));
    }

    #[test]
    fn green_converts_to_hue_120() {
        let hsva: Hsva = Color::from_rgb(0.0, 1.0, 0.0).into();
        assert!(approx(hsva.hue, 120.0));
        assert!(approx(hsva.saturation, 1.0));
        assert!(approx(hsva.value, 1.0));
    }

    #[test]
    fn blue_converts_to_hue_240() {
        let hsva: Hsva = Color::from_rgb(0.0, 0.0, 1.0).into();
        assert!(approx(hsva.hue, 240.0));
    }

    #[test]
    fn white_has_zero_saturation() {
        let hsva: Hsva = Color::from_rgb(1.0, 1.0, 1.0).into();
        assert!(approx(hsva.saturation, 0.0));
        assert!(approx(hsva.value, 1.0));
    }

    #[test]
    fn black_has_zero_saturation_and_value() {
        let hsva: Hsva = Color::from_rgb(0.0, 0.0, 0.0).into();
        assert!(approx(hsva.saturation, 0.0));
        assert!(approx(hsva.value, 0.0));
    }

    #[test]
    fn alpha_is_preserved_through_the_conversion() {
        let hsva: Hsva = Color::from_rgba(0.2, 0.4, 0.8, 0.3).into();
        assert!(approx(hsva.alpha, 0.3));
    }

    #[test]
    fn hsva_round_trips_for_primary_and_secondary_colours() {
        // Covers every branch of both the Color -> Hsva hue selection and
        // the Hsva -> Color h_prime match (buckets 0..=5), plus the
        // grayscale edge case where saturation is zero.
        for colour in [
            Color::from_rgb(1.0, 0.0, 0.0), // red
            Color::from_rgb(1.0, 1.0, 0.0), // yellow
            Color::from_rgb(0.0, 1.0, 0.0), // green
            Color::from_rgb(0.0, 1.0, 1.0), // cyan
            Color::from_rgb(0.0, 0.0, 1.0), // blue
            Color::from_rgb(1.0, 0.0, 1.0), // magenta
            Color::from_rgb(0.5, 0.5, 0.5), // gray
            Color::from_rgb(1.0, 1.0, 1.0), // white
            Color::from_rgb(0.0, 0.0, 0.0), // black
        ] {
            let hsva: Hsva = colour.into();
            let round_tripped: Color = hsva.into();
            assert_colour_approx(round_tripped, colour);
        }
    }

    #[test]
    fn hsva_to_color_clamps_out_of_range_saturation_and_value() {
        let hsva = Hsva {
            hue: 0.0,
            saturation: 2.0,
            value: 2.0,
            alpha: 1.0,
        };
        let colour: Color = hsva.into();
        assert!(colour.r <= 1.0 && colour.g <= 1.0 && colour.b <= 1.0);
    }
}
