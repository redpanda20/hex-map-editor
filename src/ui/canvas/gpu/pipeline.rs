//! The [`shader::Pipeline`] that actually renders a [`super::HexMapPrimitive`].
//!
//! There are two wgpu render pipelines here - one for solid-colour meshes
//! (`mesh.wgsl`) and one for textured quads (`image.wgsl`) - both sharing
//! the same camera uniform. Everything below turns the CPU-side
//! `DrawCommand`s built by `GpuRenderTarget` into GPU vertex buffers plus
//! an ordered list of draw calls, and then issues them.

use std::sync::Arc;

use iced::{wgpu, widget::shader};

use crate::domain::id::ImageId;

use super::geometry::{ImageVertex, MeshVertex};
use super::primitive::DrawCommand;
use super::texture::TextureCache;

const MESH_SHADER: &str = include_str!("mesh.wgsl");
const IMAGE_SHADER: &str = include_str!("image.wgsl");

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    scale_offset: [f32; 4],
}

/// A single GPU draw call, with its vertex buffer already uploaded.
enum DrawCall {
    Mesh { buffer: wgpu::Buffer, count: u32 },
    Image { buffer: wgpu::Buffer, image: ImageId },
}

pub struct HexMapPipeline {
    mesh_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    textures: TextureCache,

    // `base` draw calls are rebuilt only when the `Arc` they were built
    // from changes (see `upload`); `overlay` is cheap and rebuilt every
    // frame, so it isn't worth caching.
    last_base: Option<Arc<Vec<DrawCommand>>>,
    base_draw_calls: Vec<DrawCall>,
    overlay_draw_calls: Vec<DrawCall>,
}

impl shader::Pipeline for HexMapPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hexmap-camera-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hexmap-camera-buffer"),
            size: std::mem::size_of::<CameraUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hexmap-camera-bind-group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let textures = TextureCache::new(device);

        let mesh_pipeline = build_mesh_pipeline(device, format, &camera_bind_group_layout);
        let image_pipeline = build_image_pipeline(
            device,
            format,
            &camera_bind_group_layout,
            textures.layout(),
        );

        Self {
            mesh_pipeline,
            image_pipeline,
            camera_buffer,
            camera_bind_group,
            textures,
            last_base: None,
            base_draw_calls: Vec::new(),
            overlay_draw_calls: Vec::new(),
        }
    }
}

impl HexMapPipeline {
    pub fn update_camera(&self, queue: &wgpu::Queue, scale_offset: [f32; 4]) {
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&CameraUniform { scale_offset }),
        );
    }

    /// Rebuilds GPU-side draw calls for `base` and `overlay`.
    ///
    /// `base` is skipped whenever it's the *same `Arc`* as last frame -
    /// `HexCanvas::draw` only builds a new one when the scene's revision
    /// changes, so on a pure pan/zoom/hover frame this reuses last frame's
    /// buffers untouched. `overlay` has no such cache: it depends on the
    /// cursor position, which can change every frame.
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        base: &Arc<Vec<DrawCommand>>,
        overlay: &Arc<Vec<DrawCommand>>,
    ) {
        let base_unchanged = self
            .last_base
            .as_ref()
            .is_some_and(|prev| Arc::ptr_eq(prev, base));

        if !base_unchanged {
            self.base_draw_calls = self.build_draw_calls(device, queue, base);
            self.last_base = Some(Arc::clone(base));
        }

        self.overlay_draw_calls = self.build_draw_calls(device, queue, overlay);
    }

    fn build_draw_calls(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        commands: &[DrawCommand],
    ) -> Vec<DrawCall> {
        let max_verts = max_vertices_per_buffer::<MeshVertex>(device);
        let mut calls = Vec::with_capacity(commands.len());

        for command in commands {
            match command {
                DrawCommand::Mesh(vertices) => {
                    if vertices.is_empty() {
                        continue;
                    }
                    // `TriangleList` means every group of 3 vertices is an
                    // independent triangle, so a vertex list can be split
                    // across multiple buffers at any multiple-of-3
                    // boundary without corrupting the geometry - it
                    // doesn't need to land on hex boundaries.
                    for chunk in vertices.chunks(max_verts) {
                        let buffer =
                            upload_vertices(device, queue, "hexmap-mesh-vertices", chunk);
                        calls.push(DrawCall::Mesh {
                            buffer,
                            count: chunk.len() as u32,
                        });
                    }
                }
                DrawCommand::Image {
                    image,
                    vertices,
                    raw,
                } => {
                    self.textures.get_or_create(device, queue, *image, raw);
                    let buffer =
                        upload_vertices(device, queue, "hexmap-image-vertices", vertices);
                    calls.push(DrawCall::Image {
                        buffer,
                        image: *image,
                    });
                }
            }
        }

        calls
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        for call in self.base_draw_calls.iter().chain(&self.overlay_draw_calls) {
            match call {
                DrawCall::Mesh { buffer, count } => {
                    render_pass.set_pipeline(&self.mesh_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, buffer.slice(..));
                    render_pass.draw(0..*count, 0..1);
                }
                DrawCall::Image { buffer, image } => {
                    let Some(bind_group) = self.textures.bind_group(*image) else {
                        continue;
                    };
                    render_pass.set_pipeline(&self.image_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, bind_group, &[]);
                    render_pass.set_vertex_buffer(0, buffer.slice(..));
                    render_pass.draw(0..6, 0..1);
                }
            }
        }
    }
}

/// The largest whole number of vertices that fit in one `V`-typed vertex
/// buffer on `device`, rounded down to a multiple of 3 (so it's always
/// safe to split a `TriangleList` at that boundary).
///
/// Guards the same WebGPU buffer-size limit that `TextureCache` clamps
/// images against, just for vertex buffers instead of textures.
fn max_vertices_per_buffer<V>(device: &wgpu::Device) -> usize {
    let max_bytes = device.limits().max_buffer_size;
    let per_vertex = std::mem::size_of::<V>() as u64;
    // Clamp to u32::MAX before the `as usize` below: `usize` is only
    // guaranteed to be 32 bits wide (true on wasm32), while
    // `max_buffer_size` is a `u64` that could in principle exceed that on
    // a native 64-bit build with a very generous device.
    let verts = (max_bytes / per_vertex).clamp(3, u32::MAX as u64);
    (verts - verts % 3) as usize
}

fn upload_vertices<V: bytemuck::Pod>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    vertices: &[V],
) -> wgpu::Buffer {
    let bytes = bytemuck::cast_slice(vertices);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buffer, 0, bytes);
    buffer
}

/// Shared knobs for building one of the two render pipelines below - the
/// only things that actually differ between the mesh and image pipelines.
struct PipelineSpec<'a> {
    label: &'static str,
    shader_source: &'static str,
    bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
    vertex_buffer_layout: wgpu::VertexBufferLayout<'a>,
}

/// Builds a `TriangleList`, alpha-blended render pipeline from `spec`.
///
/// Mesh and image rendering only differ in their shader, vertex layout,
/// and bind group layouts - fragment/primitive/multisample state is
/// identical, so it's factored out here instead of being duplicated
/// between `build_mesh_pipeline` and `build_image_pipeline`.
fn build_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    spec: PipelineSpec,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(spec.label),
        source: wgpu::ShaderSource::Wgsl(spec.shader_source.into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(spec.label),
        bind_group_layouts: spec.bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(spec.label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[spec.vertex_buffer_layout],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn build_mesh_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    camera_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    build_pipeline(
        device,
        format,
        PipelineSpec {
            label: "hexmap-mesh-pipeline",
            shader_source: MESH_SHADER,
            bind_group_layouts: &[camera_layout],
            vertex_buffer_layout: wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        shader_location: 1,
                    },
                ],
            },
        },
    )
}

fn build_image_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    camera_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    build_pipeline(
        device,
        format,
        PipelineSpec {
            label: "hexmap-image-pipeline",
            shader_source: IMAGE_SHADER,
            bind_group_layouts: &[camera_layout, texture_layout],
            vertex_buffer_layout: wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<ImageVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32,
                        offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                        shader_location: 2,
                    },
                ],
            },
        },
    )
}
