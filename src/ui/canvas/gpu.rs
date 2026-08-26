//! GPU-side primitive & pipeline backing the hex map [`shader`] widget.

use std::collections::HashMap;
use std::sync::Arc;

use iced::{Color, Point, Rectangle, Vector, wgpu, widget::shader};

use crate::domain::id::ImageId;

pub const MESH_SHADER: &str = include_str!("mesh.wgsl");
pub const IMAGE_SHADER: &str = include_str!("image.wgsl");

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub opacity: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    scale_offset: [f32; 4],
}

/// Raw RGBA8 pixels for an image not yet uploaded to the GPU.
/// Pixel format: (Rgba8Unorm)
///
/// Built once per `ImageId`; Then the texture cache is reused.
/// See `GpuRenderTarget::draw_image`.
#[derive(Debug)]
pub struct RawImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// A batch of draw commands that will preserve draw order.
#[derive(Debug)]
pub enum DrawCommand {
    /// A batch of solid geometry (filled tiles, strokes, & cursor hover outline).
    /// One `MeshVertex` per layer.
    Mesh(Vec<MeshVertex>),
    /// A single textured quad.
    Image {
        image: ImageId,
        vertices: [ImageVertex; 6],
        raw: Arc<RawImage>,
    },
}

/// All resources needed to draw a frame.
/// Heavy geometry is stored with `Arc` for reuse; Base is invalidated when
/// underlying scene changes, overlay is invalidated when camera moves.
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
        // Calculate world space -> Clip space.
        let scale_x = self.zoom * 2.0 / bounds.width;
        let scale_y = -self.zoom * 2.0 / bounds.height;
        let offset_x = self.translation.x * 2.0 / bounds.width - 1.0;
        let offset_y = 1.0 - self.translation.y * 2.0 / bounds.height;

        pipeline.update_camera(queue, [scale_x, scale_y, offset_x, offset_y]);
        pipeline.upload(device, queue, &self.base, &self.overlay);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        pipeline.draw(render_pass);
        true
    }
}

enum DrawCall {
    Mesh {
        buffer: wgpu::Buffer,
        count: u32,
    },
    Image {
        buffer: wgpu::Buffer,
        image: ImageId,
    },
}

struct CachedTexture {
    bind_group: wgpu::BindGroup,
}

pub struct HexMapPipeline {
    mesh_pipeline: wgpu::RenderPipeline,
    image_pipeline: wgpu::RenderPipeline,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    texture_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    textures: HashMap<ImageId, CachedTexture>,

    // Persisted, only rebuilt when `base` changes (compared by Arc pointer).
    last_base: Option<Arc<Vec<DrawCommand>>>,
    base_draw_calls: Vec<DrawCall>,
    // Rebuilt every frame.
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hexmap-texture-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hexmap-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,

            ..Default::default()
        });

        let mesh_pipeline = build_mesh_pipeline(device, format, &camera_bind_group_layout);
        let image_pipeline = build_image_pipeline(
            device,
            format,
            &camera_bind_group_layout,
            &texture_bind_group_layout,
        );

        Self {
            mesh_pipeline,
            image_pipeline,
            camera_buffer,
            camera_bind_group,
            texture_bind_group_layout,
            sampler,
            textures: HashMap::new(),
            last_base: None,
            base_draw_calls: Vec::new(),
            overlay_draw_calls: Vec::new(),
        }
    }
}

impl HexMapPipeline {
    fn update_camera(&self, queue: &wgpu::Queue, scale_offset: [f32; 4]) {
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&CameraUniform { scale_offset }),
        );
    }

    fn upload(
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

        // Overlay is uncached
        self.overlay_draw_calls = self.build_draw_calls(device, queue, overlay);
    }

    fn build_draw_calls(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        commands: &[DrawCommand],
    ) -> Vec<DrawCall> {
        let mut calls = Vec::with_capacity(commands.len());

        for command in commands {
            match command {
                DrawCommand::Mesh(vertices) => {
                    if vertices.is_empty() {
                        continue;
                    }
                    let buffer = upload_vertices(device, queue, "hexmap-mesh-vertices", vertices);
                    calls.push(DrawCall::Mesh {
                        buffer,
                        count: vertices.len() as u32,
                    });
                }
                DrawCommand::Image {
                    image,
                    vertices,
                    raw,
                } => {
                    self.ensure_texture(device, queue, *image, raw);
                    let buffer = upload_vertices(device, queue, "hexmap-image-vertices", vertices);
                    calls.push(DrawCall::Image {
                        buffer,
                        image: *image,
                    });
                }
            }
        }

        calls
    }

    fn ensure_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: ImageId,
        raw: &RawImage,
    ) {
        if self.textures.contains_key(&id) {
            return;
        }

        let size = wgpu::Extent3d {
            width: raw.width,
            height: raw.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hexmap-image-texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &raw.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * raw.width),
                rows_per_image: Some(raw.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hexmap-image-bind-group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.textures.insert(id, CachedTexture { bind_group });
    }

    fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        for call in self.base_draw_calls.iter().chain(&self.overlay_draw_calls) {
            match call {
                DrawCall::Mesh { buffer, count } => {
                    render_pass.set_pipeline(&self.mesh_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, buffer.slice(..));
                    render_pass.draw(0..*count, 0..1);
                }
                DrawCall::Image { buffer, image } => {
                    let Some(texture) = self.textures.get(image) else {
                        continue;
                    };
                    render_pass.set_pipeline(&self.image_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, &texture.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, buffer.slice(..));
                    render_pass.draw(0..6, 0..1);
                }
            }
        }
    }
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

fn build_mesh_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    camera_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("hexmap-mesh-shader"),
        source: wgpu::ShaderSource::Wgsl(MESH_SHADER.into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("hexmap-mesh-pipeline-layout"),
        bind_group_layouts: &[camera_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("hexmap-mesh-pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
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
            }],
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

fn build_image_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    camera_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("hexmap-image-shader"),
        source: wgpu::ShaderSource::Wgsl(IMAGE_SHADER.into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("hexmap-image-pipeline-layout"),
        bind_group_layouts: &[camera_layout, texture_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("hexmap-image-pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
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
            }],
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

fn hex_corner(i: usize, size: f32) -> [f32; 2] {
    let angle = std::f32::consts::PI / 180.0 * (60.0 * i as f32);
    [size * angle.cos(), size * angle.sin()]
}

fn color_to_array(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

/// Appends a filled hex (triangle fan from the center) to `out`.
pub fn push_hex_fill(out: &mut Vec<MeshVertex>, center: Point, size: f32, color: Color) {
    let col = color_to_array(color);
    let corners: [[f32; 2]; 6] = std::array::from_fn(|i| hex_corner(i, size));

    for i in 0..6 {
        let a = corners[i];
        let b = corners[(i + 1) % 6];
        out.push(MeshVertex {
            position: [center.x, center.y],
            color: col,
        });
        out.push(MeshVertex {
            position: [center.x + a[0], center.y + a[1]],
            color: col,
        });
        out.push(MeshVertex {
            position: [center.x + b[0], center.y + b[1]],
            color: col,
        });
    }
}

/// Create a stroke with `width` for a hexagon with a given `size` and `center`.
///
/// `width` is in world space units; for constant on screen thickness use
/// `desired_screen_width / zoom`
pub fn push_hex_stroke(
    out: &mut Vec<MeshVertex>,
    center: Point,
    size: f32,
    color: Color,
    width: f32,
) {
    let col = color_to_array(color);
    let inner = size - width * 0.5;
    let outer = size + width * 0.5;

    for i in 0..6 {
        let dir_a = hex_corner(i, 1.0);
        let dir_b = hex_corner((i + 1) % 6, 1.0);

        let a_out = [center.x + dir_a[0] * outer, center.y + dir_a[1] * outer];
        let b_out = [center.x + dir_b[0] * outer, center.y + dir_b[1] * outer];
        let a_in = [center.x + dir_a[0] * inner, center.y + dir_a[1] * inner];
        let b_in = [center.x + dir_b[0] * inner, center.y + dir_b[1] * inner];

        out.push(MeshVertex {
            position: a_in,
            color: col,
        });
        out.push(MeshVertex {
            position: a_out,
            color: col,
        });
        out.push(MeshVertex {
            position: b_out,
            color: col,
        });

        out.push(MeshVertex {
            position: a_in,
            color: col,
        });
        out.push(MeshVertex {
            position: b_out,
            color: col,
        });
        out.push(MeshVertex {
            position: b_in,
            color: col,
        });
    }
}

/// Builds the 6 vertices (2 triangles) for an image quad covering `bounds`.
pub fn quad_vertices(bounds: Rectangle, opacity: f32) -> [ImageVertex; 6] {
    let (x0, y0) = (bounds.x, bounds.y);
    let (x1, y1) = (bounds.x + bounds.width, bounds.y + bounds.height);

    let tl = ImageVertex {
        position: [x0, y0],
        uv: [0.0, 0.0],
        opacity,
    };
    let tr = ImageVertex {
        position: [x1, y0],
        uv: [1.0, 0.0],
        opacity,
    };
    let bl = ImageVertex {
        position: [x0, y1],
        uv: [0.0, 1.0],
        opacity,
    };
    let br = ImageVertex {
        position: [x1, y1],
        uv: [1.0, 1.0],
        opacity,
    };

    [tl, tr, br, tl, br, bl]
}
