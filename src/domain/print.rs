//! Physical parameters for printing

use iced::{Rectangle, Size};

/// Points (1/72 inch) per centimetre.
/// Typographic standard.
pub const POINTS_PER_CM: f32 = 72.0 / 2.54;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalScale {
    /// Physical distance measured edge-to-edge
    pub hex_height_cm: f32,
}

impl PhysicalScale {
    /// Points per logical unit.
    pub fn points_per_unit(&self) -> f32 {
        self.hex_height_cm.max(f32::EPSILON) * POINTS_PER_CM / 3.0_f32.sqrt()
    }

    pub fn hex_height_points(&self) -> f32 {
        self.hex_height_cm * POINTS_PER_CM
    }
}

/// Paper sizes a print export can be tiled across (portrait; width <= height).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageSize {
    A4,
    A3,
    UsLetter,
    UsLegal,
    UsTabloid,
}

impl PageSize {
    /// Page size in points (1/72 inch), portrait orientation.
    pub const fn size_points(self) -> (f32, f32) {
        match self {
            PageSize::A4 => (595.28, 841.89),
            PageSize::A3 => (841.89, 1190.55),
            PageSize::UsLetter => (612.0, 792.0),
            PageSize::UsLegal => (612.0, 1008.0),
            PageSize::UsTabloid => (792.0, 1224.0),
        }
    }
}

/// A margin, in centimetres, reserved on every edge of every page. Printers
/// generally can't print edge-to-edge, so content placed in that dead zone
/// would either be clipped outright or - if the print driver compensates by
/// scaling the page down to fit - thrown off the fixed hex scale. Reserving
/// the margin at generation time keeps both the scale and the content intact
/// regardless of what the printer does with its unprintable border.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageMargin {
    pub cm: f32,
}

impl PageMargin {
    pub const NONE: PageMargin = PageMargin { cm: 0.0 };

    pub fn points(self) -> f32 {
        self.cm.max(0.0) * POINTS_PER_CM
    }
}

/// A columns x rows grid of pages a map tiles into. Computing this only
/// needs the map's bounding box - already known cheaply, since
/// `Renderable::bounds` never renders anything - so it's suitable for a live
/// preview ("this will print as 3 x 2 pages") without drawing a single hex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageGrid {
    pub columns: u32,
    pub rows: u32,
}

impl PageGrid {
    pub fn total(&self) -> u32 {
        self.columns * self.rows
    }
}

/// The full set of physical parameters a scene is printed with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrintSettings {
    pub scale: PhysicalScale,
    pub page_size: PageSize,
    pub margin: PageMargin,
}

impl PrintSettings {
    /// 1 hex = 2 cm, on A4, with a 1.5 cm margin.
    pub const DEFAULT: PrintSettings = PrintSettings {
        scale: PhysicalScale { hex_height_cm: 2.0 },
        page_size: PageSize::A4,
        margin: PageMargin { cm: 1.5 },
    };

    /// The page's usable content area, in points - the page with the margin
    /// removed from every edge.
    ///
    /// Clamped well above zero so a margin close to (or larger than) the
    /// page can't produce a zero or negative tile and send tiling into an
    /// infinite/NaN loop.
    fn content_size_points(&self) -> (f32, f32) {
        const MIN_CONTENT_PT: f32 = 1.0;
        let (page_w, page_h) = self.page_size.size_points();
        let margin = 2.0 * self.margin.points();
        (
            (page_w - margin).max(MIN_CONTENT_PT),
            (page_h - margin).max(MIN_CONTENT_PT),
        )
    }

    /// The size, in logical units, of a single page's content area.
    pub fn tile_size(&self) -> Size {
        let (w, h) = self.content_size_points();
        let ppu = self.scale.points_per_unit();
        Size::new(w / ppu, h / ppu)
    }

    /// How many pages (in a columns x rows grid) `bounds` (in logical units)
    /// tiles into at these settings.
    pub fn page_grid(&self, bounds: Rectangle) -> PageGrid {
        let tile = self.tile_size();
        PageGrid {
            columns: (bounds.width / tile.width).ceil().max(1.0) as u32,
            rows: (bounds.height / tile.height).ceil().max(1.0) as u32,
        }
    }
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use iced::Point;

    use super::*;

    const TEST_SCALE: PhysicalScale = PhysicalScale { hex_height_cm: 2.0 };

    #[test]
    fn default_is_2cm_hexes_on_a4_with_a_1_5cm_margin() {
        let settings = PrintSettings::default();
        assert_eq!(settings.scale.hex_height_cm, 2.0);
        assert_eq!(settings.page_size, PageSize::A4);
        assert_eq!(settings.margin.cm, 1.5);
    }

    #[test]
    fn margin_reduces_usable_page_area_so_more_pages_are_needed() {
        // At 2cm/hex on A4, a 14x14-unit map fits a single page with no
        // margin (a full A4 page is ~14.57 units wide at this scale).
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(14.0, 14.0));

        let no_margin = PrintSettings {
            scale: TEST_SCALE,
            page_size: PageSize::A4,
            margin: PageMargin::NONE,
        };
        assert_eq!(
            no_margin.page_grid(bounds),
            PageGrid {
                columns: 1,
                rows: 1
            }
        );

        // A generous margin eats enough of the page that the same map now
        // needs more than one sheet.
        let with_margin = PrintSettings {
            margin: PageMargin { cm: 3.0 },
            ..no_margin
        };
        assert!(with_margin.page_grid(bounds).total() > no_margin.page_grid(bounds).total());
    }

    #[test]
    fn margin_never_produces_a_degenerate_tile() {
        let settings = PrintSettings {
            scale: TEST_SCALE,
            page_size: PageSize::A4,
            margin: PageMargin { cm: 1000.0 },
        };
        let tile = settings.tile_size();
        assert!(tile.width > 0.0 && tile.width.is_finite());
        assert!(tile.height > 0.0 && tile.height.is_finite());
    }

    #[test]
    fn zero_scale_does_not_divide_by_zero() {
        let settings = PrintSettings {
            scale: PhysicalScale { hex_height_cm: 0.0 },
            page_size: PageSize::A4,
            margin: PageMargin::NONE,
        };
        let tile = settings.tile_size();
        assert!(tile.width.is_finite() && tile.height.is_finite());
    }
}
