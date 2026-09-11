use std::sync::Arc;

use iced::{Color, Point, Rectangle};

use crate::domain::{HexCoord, RenderTarget, assets::AssetStore, id::ImageId};

use super::HEX_SIZE;
use super::gpu::{self, DrawCommand};

/// Adapter for `RenderTarget` into [`gpu::DrawCommand`]s.
///
/// Preserves draw order, so that images can be painted over.
pub(super) struct GpuRenderTarget<'a> {
    bounds: Rectangle,
    assets: &'a AssetStore,
    commands: Vec<DrawCommand>,
    current_mesh: Vec<gpu::MeshVertex>,
}

impl<'a> GpuRenderTarget<'a> {
    pub(super) fn new(bounds: Rectangle, assets: &'a AssetStore) -> Self {
        Self {
            bounds,
            assets,
            commands: Vec::new(),
            current_mesh: Vec::new(),
        }
    }

    /// Flushes any buffered mesh vertices into a `DrawCommand::Mesh`.
    ///
    /// Called before an image command (so the image lands between the
    /// mesh geometry drawn before and after it, preserving layer order)
    /// and once more at the end in `finish`.
    fn flush_mesh(&mut self) {
        if !self.current_mesh.is_empty() {
            self.commands
                .push(DrawCommand::Mesh(std::mem::take(&mut self.current_mesh)));
        }
    }

    pub(super) fn finish(mut self) -> Vec<DrawCommand> {
        self.flush_mesh();
        self.commands
    }
}

impl RenderTarget for GpuRenderTarget<'_> {
    fn hex_to_point(&self, coord: &HexCoord) -> Point {
        let point = coord.to_cartesian();
        Point::new(point.x * HEX_SIZE, point.y * HEX_SIZE)
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }

    fn fill_polygon(&mut self, point: &Point, fill: Color) {
        gpu::push_hex_fill(&mut self.current_mesh, *point, HEX_SIZE, fill);
    }

    fn stroke_polygon(&mut self, point: &Point, colour: Color, width: f32) {
        gpu::push_hex_stroke(&mut self.current_mesh, *point, HEX_SIZE, colour, width);
    }

    fn draw_image(&mut self, bounds: Rectangle, image: ImageId, opacity: f32) {
        self.flush_mesh();

        let Some(handle) = self.assets.image_data(image) else {
            return;
        };

        let raw = match handle {
            iced::advanced::image::Handle::Rgba {
                width,
                height,
                pixels,
                ..
            } => Arc::new(gpu::RawImage {
                width: *width,
                height: *height,
                pixels: pixels.as_ref().to_vec(),
            }),
            _ => return,
        };

        self.commands.push(DrawCommand::Image {
            image,
            vertices: gpu::quad_vertices(bounds, opacity),
            raw,
        });
    }
}
