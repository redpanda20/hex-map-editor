//! Implements [`RenderTarget`] for PDF exports.

use iced::{Color, Point, Rectangle};
use pdf_writer::Content;

use crate::domain::{
    HexCoord, RenderTarget, assets::AssetStore, id::ImageId, layer::image::EDITOR_HEX_SIZE,
    print::PrintSettings,
};

use super::document::{PdfDocument, alpha_name, image_name};

/// Width of the hex grid lines, in points.
const GRID_LINE_WIDTH_PT: f32 = 0.5;

/// Width of the outline of opaque fills, in points.
/// Outline covers seams between hexes.
const SEAM_WIDTH_PT: f32 = 0.25;

/// A single page's worth of drawing.
pub struct PdfPageTarget<'a> {
    doc: &'a mut PdfDocument,
    assets: &'a AssetStore,
    content: Content,
    /// This page's viewport, in logical units.
    bounds: Rectangle,
    settings: PrintSettings,
    current_alpha: u8,
    /// Images actually drawn on this page, in draw order, and deduplicated.
    used_images: Vec<usize>,
}

impl<'a> PdfPageTarget<'a> {
    pub(super) fn new(
        doc: &'a mut PdfDocument,
        assets: &'a AssetStore,
        bounds: Rectangle,
        settings: PrintSettings,
        page_size_pt: (f32, f32),
    ) -> Self {
        let mut content = Content::new();
        let s = settings.scale.points_per_unit();
        let margin_pt = settings.margin.points();
        let (_, page_h_pt) = page_size_pt;

        // Map logical units to page space
        // Scale,
        // flip Y (PDF is Y-up),
        // Translate to match origin,
        // Inset by margin.
        content.save_state();
        content.transform([
            s,
            0.0,
            0.0,
            -s,
            margin_pt - bounds.x * s,
            page_h_pt - margin_pt + bounds.y * s,
        ]);

        Self {
            doc,
            assets,
            content,
            bounds,
            settings,
            current_alpha: 255,
            used_images: Vec::new(),
        }
    }

    fn set_alpha(&mut self, alpha: f32) {
        let alpha = self.doc.alpha_key(alpha);
        if alpha != self.current_alpha {
            self.current_alpha = alpha;
            self.content.set_parameters(alpha_name(alpha).as_name());
        }
    }

    fn hex_path(&mut self, centre: &Point) {
        for i in 0..6 {
            let angle = (60.0 * i as f32).to_radians();
            let (x, y) = (centre.x + angle.cos(), centre.y + angle.sin());
            if i == 0 {
                self.content.move_to(x, y);
            } else {
                self.content.line_to(x, y);
            }
        }
        self.content.close_path();
    }

    /// Ends the page's content stream.
    ///
    /// Returns the content bytes, and the used images (deduplicated).
    pub(super) fn finish(self) -> (Vec<u8>, Vec<usize>) {
        (self.content.finish().to_vec(), self.used_images)
    }
}

impl RenderTarget for PdfPageTarget<'_> {
    fn hex_to_point(&self, coord: &HexCoord) -> Point {
        let v = coord.to_cartesian();
        Point::new(v.x, v.y)
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn fill_polygon(&mut self, point: &Point, fill: Color) {
        self.set_alpha(fill.a);
        self.content.set_fill_rgb(fill.r, fill.g, fill.b);
        self.hex_path(point);

        if fill.a >= 1.0 {
            self.content.set_stroke_rgb(fill.r, fill.g, fill.b);
            self.content
                .set_line_width(SEAM_WIDTH_PT / self.settings.scale.points_per_unit());
            self.content.fill_nonzero_and_stroke();
        } else {
            self.content.fill_nonzero();
        }
    }

    fn stroke_polygon(&mut self, point: &Point, colour: Color, _stroke_width: f32) {
        self.set_alpha(colour.a);
        self.content.set_stroke_rgb(colour.r, colour.g, colour.b);
        self.content
            .set_line_width(GRID_LINE_WIDTH_PT / self.settings.scale.points_per_unit());
        self.hex_path(point);
        self.content.stroke();
    }

    fn draw_image(&mut self, bounds: Rectangle, image_id: ImageId, opacity: f32) {
        // Image bounds are in editor pixels; convert to hex units.
        let bounds = bounds * (1.0 / EDITOR_HEX_SIZE);

        if !bounds.intersects(&self.bounds) {
            return;
        }

        let Some(index) = self.doc.image_key(self.assets, image_id) else {
            return;
        };
        if !self.used_images.contains(&index) {
            self.used_images.push(index);
        }

        self.set_alpha(opacity);
        self.content.save_state();
        // Page transform flips Y, so flip again to keep image upright.
        self.content.transform([
            bounds.width,
            0.0,
            0.0,
            -bounds.height,
            bounds.x,
            bounds.y + bounds.height,
        ]);
        self.content.x_object(image_name(index).as_name());
        self.content.restore_state();
    }
}
