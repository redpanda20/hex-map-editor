//! wgpu adapter for the hex map [`shader`](iced::widget::shader) widget.
//!
//! This module is the only place in the codebase that talks to `wgpu`
//! directly. It's split by concern rather than kept as one file:
//!
//! - [`geometry`]  - Pure CPU math: hex/quad vertices, no `wgpu` types.
//! - [`texture`]   - GPU texture cache, per-`ImageId`.
//! - [`pipeline`]  - Render pipelines (mesh, image) and the
//!   per-frame draw-call bookkeeping that uses `texture`.
//! - [`primitive`] - `shader::Primitive` adapter (`HexMapPrimitive`)
//!   that iced calls into each frame, plus `DrawCommand`, the batched
//!   draw-op type that `super::render_target::GpuRenderTarget` builds and
//!   `pipeline` consumes.

mod geometry;
mod pipeline;
mod primitive;
mod texture;

pub use geometry::{MeshVertex, RawImage, push_hex_fill, push_hex_stroke, quad_vertices};
pub use primitive::{DrawCommand, HexMapPrimitive};
