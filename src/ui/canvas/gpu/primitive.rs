//! The boundary between iced's `shader` widget and our wgpu pipeline.
//!
//! [`HexMapPrimitive`] is what `HexCanvas::draw` (a `shader::Program`)
//! hands back each frame; [`shader::Primitive::prepare`]/`draw` are where
//! iced calls into it with a live `wgpu::Device`/`Queue`/`RenderPass`. This
//! is the only file that needs to know both "iced shader widget" types and
//! "our pipeline" types - everything pipeline-internal lives in
//! [`super::pipeline`].

use std::sync::Arc;

use iced::{Rectangle, Vector, wgpu, widget::shader};

use crate::domain::id::ImageId;

use super::geometry::{ImageVertex, MeshVertex, SharedRawImage};
use super::pipeline::HexMapPipeline;

/// One batched draw operation, in the order it should be drawn.
///
/// Built by `GpuRenderTarget` (the `RenderTarget` impl layers draw into)
/// and consumed by `HexMapPipeline::build_draw_calls`.
#[derive(Debug)]
pub enum DrawCommand {
    /// A batch of solid geometry: filled tiles, hex-grid strokes, the
    /// cursor-hover outline. One `MeshVertex` triple per triangle.
    Mesh(Vec<MeshVertex>),
    /// A single textured quad.
    Image {
        image: ImageId,
        vertices: [ImageVertex; 6],
        raw: SharedRawImage,
    },
}

/// Everything needed to draw one frame of the hex map.
///
/// `base` (the map's layers) and `overlay` (grid lines + cursor hover) are
/// tracked separately and `Arc`-shared so `HexMapPipeline::upload` can
/// skip re-uploading `base`'s GPU buffers on frames where only the camera
/// or cursor moved - see `HexCanvas::draw`, which is what actually decides
/// when `base` gets rebuilt.
#[derive(Debug, Clone)]
pub struct HexMapPrimitive {
    pub base: Arc<Vec<DrawCommand>>,
    pub overlay: Arc<Vec<DrawCommand>>,
    pub translation: Vector,
    pub zoom: f32,
}

impl shader::Primitive for HexMapPrimitive {
    type Pipeline = HexMapPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        pipeline.update_camera(queue, camera_scale_offset(self.translation, self.zoom, bounds));
        pipeline.upload(device, queue, &self.base, &self.overlay);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        pipeline.draw(render_pass);
        true
    }
}

/// Builds the `[scale_x, scale_y, offset_x, offset_y]` uniform that
/// `mesh.wgsl`/`image.wgsl` use to map "pixel-world" vertex positions
/// (world space, per `HEX_SIZE` - see the `canvas` module docs) into clip
/// space, folding in the current pan (`translation`) and `zoom`.
///
/// Y is flipped because wgpu clip space has +Y pointing up, while our
/// widget/world space (like most 2D UI coordinates) has +Y pointing down.
fn camera_scale_offset(translation: Vector, zoom: f32, bounds: &Rectangle) -> [f32; 4] {
    let scale_x = zoom * 2.0 / bounds.width;
    let scale_y = -zoom * 2.0 / bounds.height;
    let offset_x = translation.x * 2.0 / bounds.width - 1.0;
    let offset_y = 1.0 - translation.y * 2.0 / bounds.height;

    [scale_x, scale_y, offset_x, offset_y]
}
