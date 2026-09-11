//! CPU-side vertex construction.

use std::borrow::Cow;
use std::sync::Arc;

use iced::{Color, Rectangle};

/// One vertex of a solid-colour mesh triangle.
///
/// Filled hexes, stroke outlines, the cursor-hover ring.
/// Matches the `VertexInput` in `mesh.wgsl`.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

/// One vertex of a textured quad (image layers).
///
/// Matches the `VertexInput` in `image.wgsl`.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub opacity: f32,
}

/// Raw RGBA8 pixels for an image not yet uploaded to the GPU.
///
/// Built once per `ImageId` by `GpuRenderTarget::draw_image`;
/// then cached for all future uses - see `TextureCache::get_or_create`.
#[derive(Debug)]
pub struct RawImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

fn color_to_array(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

/// The `i`th corner (of 6, going clockwise) of a flat-top hex of the given
/// `size`, relative to its center.
fn hex_corner(i: usize, size: f32) -> [f32; 2] {
    let angle = std::f32::consts::PI / 180.0 * (60.0 * i as f32);
    [size * angle.cos(), size * angle.sin()]
}

/// Appends a filled hex (a 6-triangle fan from the center) to `out`.
pub fn push_hex_fill(out: &mut Vec<MeshVertex>, center: iced::Point, size: f32, color: Color) {
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

/// Appends a hex outline (6 quads, 2 triangles each) to `out`.
///
/// `width` is in world space units; for a constant on-screen thickness use
/// `desired_screen_width / zoom`.
pub fn push_hex_stroke(
    out: &mut Vec<MeshVertex>,
    center: iced::Point,
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

/// Builds the 6 vertices (2 triangles) of an image quad covering `bounds`.
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

/// Downscales `raw`, if necessary, to fit within a device's
/// `max_texture_dimension_2d`, preserving aspect ratio.
///
/// Returns borrowed pixels when no resize was needed, to avoid a copy on
/// the common path.
pub fn clamp_raw_image(raw: &RawImage, max_dim: u32) -> (u32, u32, Cow<'_, [u8]>) {
    if raw.width <= max_dim && raw.height <= max_dim {
        return (raw.width, raw.height, Cow::Borrowed(&raw.pixels));
    }

    let scale = (max_dim as f32 / raw.width as f32).min(max_dim as f32 / raw.height as f32);
    let new_width = ((raw.width as f32 * scale).floor() as u32).max(1);
    let new_height = ((raw.height as f32 * scale).floor() as u32).max(1);

    let Some(buffer) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        raw.width,
        raw.height,
        raw.pixels.as_slice(),
    ) else {
        // Should never happen, but better to log the problem and continue.
        eprintln!("Error: Expected length of `pixels` to be width * height * 4");
        return (1, 1, Cow::Owned(vec![0, 0, 0, 0]));
    };

    let resized = image::imageops::resize(
        &buffer,
        new_width,
        new_height,
        image::imageops::FilterType::Triangle,
    );

    (new_width, new_height, Cow::Owned(resized.into_raw()))
}

/// Arc-wraps a `RawImage`; kept as a small alias so call sites (and this
/// file) read `SharedRawImage` instead of a bare `Arc<RawImage>`.
pub type SharedRawImage = Arc<RawImage>;
