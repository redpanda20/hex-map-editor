// Solid-color triangle mesh shader.
//
// Shares the exact same `camera` uniform/layout convention as image.wgsl;
// So that both can be interleaved, with a shared "world space".
//
// Hex coordinates are pre-multiplied by HEX_SIZE, but camera projection
// has not been applied (pan, zoom, world space -> clip space).

struct Camera {
    scale_offset: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let x = input.position.x * camera.scale_offset.x + camera.scale_offset.z;
    let y = input.position.y * camera.scale_offset.y + camera.scale_offset.w;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
