// Textured quad shader, used for image layers.
//
// Shares the exact same `camera` uniform/layout convention as mesh.wgsl;
// So that both can be interleaved, with a shared "world space".

struct Camera {
    scale_offset: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var image_texture: texture_2d<f32>;

@group(1) @binding(1)
var image_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) opacity: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) opacity: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let x = input.position.x * camera.scale_offset.x + camera.scale_offset.z;
    let y = input.position.y * camera.scale_offset.y + camera.scale_offset.w;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = input.uv;
    out.opacity = input.opacity;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(image_texture, image_sampler, input.uv);
    return vec4<f32>(sample.rgb, sample.a * input.opacity);
}
