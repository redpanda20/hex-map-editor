//! Physical parameters for printing

use std::fmt;

use iced::{Rectangle, Size};

/// Physical size of a hex tile.
/// Measured edge to edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileSize {
    pub cm: f32,
}

impl TileSize {
    pub fn cm_per_unit(&self) -> f32 {
        self.cm.max(f32::EPSILON) / 3.0_f32.sqrt()
    }
}

/// Paper sizes a print export can be tiled across (portrait; width <= height).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageSize {
    A2,
    A3,
    A4,
}

impl PageSize {
    pub const ALL: [PageSize; 3] = [PageSize::A2, PageSize::A3, PageSize::A4];

    pub const fn size(self) -> Size {
        let (width, height) = self.size_cm();
        Size { width, height }
    }

    pub const fn size_cm(self) -> (f32, f32) {
        match self {
            PageSize::A2 => (42.0, 59.4),
            PageSize::A3 => (29.7, 42.0),
            PageSize::A4 => (21.0, 29.7),
        }
    }
}

impl fmt::Display for PageSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PageSize::A2 => "A2",
            PageSize::A3 => "A3",
            PageSize::A4 => "A4",
        })
    }
}

/// Page margin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageMargin {
    pub cm: f32,
}

impl PageMargin {
    pub const NONE: PageMargin = PageMargin { cm: 0.0 };
}

/// The full set of physical parameters a scene is printed with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrintSettings {
    pub scale: TileSize,
    pub page_size: PageSize,
    pub margin: PageMargin,
}

impl PrintSettings {
    pub const DEFAULT: PrintSettings = PrintSettings {
        scale: TileSize { cm: 2.0 },
        page_size: PageSize::A4,
        margin: PageMargin { cm: 1.5 },
    };

    /// The page's usable content area.
    ///
    /// Clamped at 1 so that excessive margins can't produce zero or negative tilings.
    fn content_size(&self) -> Size {
        const MIN_CONTENT_CM: f32 = 1.0;
        let margin = 2.0 * self.margin.cm;
        let (width, height) = self.page_size.size_cm();

        let width = (width - margin).max(MIN_CONTENT_CM);
        let height = (height - margin).max(MIN_CONTENT_CM);
        Size { width, height }
    }

    /// The size, in logical units, of a single page's content area.
    pub fn tile_size(&self) -> Size {
        self.content_size() / self.scale.cm_per_unit()
    }

    /// Calculate pages (columns x rows) required to cover `bounds`.
    pub fn page_grid(&self, bounds: Rectangle) -> Size<u32> {
        let tile = self.tile_size();
        let columns = (bounds.width / tile.width).max(1.0);
        let rows = (bounds.height / tile.height).max(1.0);

        Size {
            width: columns.ceil() as u32,
            height: rows.ceil() as u32,
        }
    }

    pub fn is_valid(&self) -> bool {
        let scale = self.scale.cm;
        let margin = self.margin.cm;
        let (width, height) = self.page_size.size_cm();

        scale > 0.0 && margin >= 0.0 && margin * 2.0 < width && margin * 2.0 < height
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

    const TEST_SCALE: TileSize = TileSize { cm: 2.0 };

    #[test]
    fn default_is_2cm_hexes_on_a4_with_a_1_5cm_margin() {
        let settings = PrintSettings::default();
        assert_eq!(settings.scale.cm, 2.0);
        assert_eq!(settings.page_size, PageSize::A4);
        assert_eq!(settings.margin.cm, 1.5);
    }

    #[test]
    fn margin_reduces_usable_page_area_so_more_pages_are_needed() {
        // With the default sizing (2cm tall, 1.15cm wide),
        // An A4 page is 18.82 x 14.85 tiles.
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(14.0, 14.0));

        let no_margin = PrintSettings {
            scale: TEST_SCALE,
            page_size: PageSize::A4,
            margin: PageMargin::NONE,
        };

        assert_eq!(no_margin.page_grid(bounds), Size::new(1, 1));

        // A wide enough margin to decrease width by 1 tile
        let with_margin = PrintSettings {
            margin: PageMargin { cm: 3.0 },
            ..no_margin
        };

        let total = |size: Size<u32>| size.width * size.height;

        assert!(total(with_margin.page_grid(bounds)) > total(no_margin.page_grid(bounds)));
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
            scale: TileSize { cm: 0.0 },
            page_size: PageSize::A4,
            margin: PageMargin::NONE,
        };
        let tile = settings.tile_size();
        assert!(tile.width.is_finite() && tile.height.is_finite());
    }
}
