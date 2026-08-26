use iced::{Color, Point, Rectangle, advanced::image::Handle};
use image::{ImageBuffer, Rgba};

use crate::domain::{HexCoord, RenderTarget, assets::AssetStore, id::ImageId};

use super::EXPORT_HEX_SIZE;

pub struct PngRenderTarget<'a> {
    image: &'a mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    bounds: Rectangle,
    assets: &'a AssetStore,
}

impl<'a> PngRenderTarget<'a> {
    pub fn new(
        image: &'a mut ImageBuffer<Rgba<u8>, Vec<u8>>,
        bounds: Rectangle,
        assets: &'a AssetStore,
    ) -> Self {
        Self {
            image,
            bounds,
            assets,
        }
    }
}

impl RenderTarget for PngRenderTarget<'_> {
    fn hex_to_point(&self, coord: &HexCoord) -> Point {
        let point = coord.to_cartesian();

        Point::new(point.x * EXPORT_HEX_SIZE, point.y * EXPORT_HEX_SIZE)
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds * (1.0 / EXPORT_HEX_SIZE)
    }

    fn fill_polygon(&mut self, point: &Point, fill: Color) {
        let centre = Point::new(point.x - self.bounds.x, point.y - self.bounds.y);

        let vertices = hex_vertices_f(centre.x, centre.y);

        fill_polygon(self.image, &vertices, fill.into_rgba8());
    }

    fn stroke_polygon(&mut self, point: &Point, colour: Color) {
        let centre = Point::new(point.x - self.bounds.x, point.y - self.bounds.y);

        let vertices = hex_vertices_f(centre.x, centre.y);

        stroke_polygon(self.image, &vertices, colour.into_rgba8());
    }

    fn draw_image(&mut self, bounds: Rectangle, image_id: ImageId, opacity: f32) {
        let Some(Handle::Rgba {
            id: _,
            width,
            height,
            pixels,
        }) = self.assets.image_data(image_id).cloned()
        else {
            return;
        };

        let Some(src) = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, pixels.into()) else {
            return;
        };

        // `bounds` is in the same coordinate system as the other rendering
        // operations, so translate it relative to the export image.
        let x = (bounds.x - self.bounds.x).round() as i64;
        let y = (bounds.y - self.bounds.y).round() as i64;

        let dst_width = bounds.width.max(0.0).round() as u32;
        let dst_height = bounds.height.max(0.0).round() as u32;

        if dst_width == 0 || dst_height == 0 {
            return;
        }

        // Resize only when necessary.
        let src = if src.width() != dst_width || src.height() != dst_height {
            image::imageops::resize(
                &src,
                dst_width,
                dst_height,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            src
        };

        let opacity = opacity.clamp(0.0, 1.0);

        for (sx, sy, pixel) in src.enumerate_pixels() {
            let dx = x + sx as i64;
            let dy = y + sy as i64;

            // Clip against the destination image.
            if dx < 0
                || dy < 0
                || dx >= self.image.width() as i64
                || dy >= self.image.height() as i64
            {
                continue;
            }

            let mut colour = pixel.0;

            // Apply the requested opacity to the source alpha.
            colour[3] = (colour[3] as f32 * opacity).round() as u8;

            if colour[3] == 0 {
                continue;
            }

            let dst = self.image.get_pixel_mut(dx as u32, dy as u32);
            blend(dst, colour);
        }
    }
}

fn hex_vertices_f(cx: f32, cy: f32) -> [(f32, f32); 6] {
    std::array::from_fn(|i| {
        let angle_rad = (60.0 * i as f32).to_radians();

        (
            cx + EXPORT_HEX_SIZE * angle_rad.cos(),
            cy + EXPORT_HEX_SIZE * angle_rad.sin(),
        )
    })
}

fn stroke_polygon(
    buf: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    vertices: &[(f32, f32)],
    colour: [u8; 4],
) {
    for i in 0..vertices.len() {
        let a = vertices[i];
        let b = vertices[(i + 1) % vertices.len()];

        draw_line(buf, a, b, colour);
    }
}

fn draw_line(
    buf: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    (x0, y0): (f32, f32),
    (x1, y1): (f32, f32),
    colour: [u8; 4],
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = dx.abs().max(dy.abs()).ceil() as usize;

    if steps == 0 {
        return;
    }

    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let x = x0 + dx * t;
        let y = y0 + dy * t;

        if x >= 0.0 && y >= 0.0 && x < buf.width() as f32 && y < buf.height() as f32 {
            buf.put_pixel(x as u32, y as u32, Rgba(colour));
        }
    }
}

fn fill_polygon(buf: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, vertices: &[(f32, f32)], color: [u8; 4]) {
    let width = buf.width() as f32;
    let height = buf.height() as f32;

    // Axis-aligned bounding box of the polygon.
    let xs: Vec<f32> = vertices.iter().map(|(x, _)| *x).collect();
    let ys: Vec<f32> = vertices.iter().map(|(_, y)| *y).collect();
    let xmin = xs.iter().cloned().fold(f32::INFINITY, f32::min).max(0.0) as u32;
    let xmax = xs
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
        .min(width - 1.0) as u32;
    let ymin = ys.iter().cloned().fold(f32::INFINITY, f32::min).max(0.0) as u32;
    let ymax = ys
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
        .min(height - 1.0) as u32;

    for py in ymin..=ymax {
        for px in xmin..=xmax {
            if point_in_polygon(px as f32 + 0.5, py as f32 + 0.5, vertices) {
                let dst = buf.get_pixel_mut(px, py);
                blend(dst, color);
            }
        }
    }
}

fn point_in_polygon(x: f32, y: f32, verticies: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let mut j = verticies.len() - 1;
    for i in 0..verticies.len() {
        let (xi, yi) = verticies[i];
        let (xj, yj) = verticies[j];
        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn blend(dst: &mut Rgba<u8>, src: [u8; 4]) {
    let sa = src[3] as f32 / 255.0;
    let da = dst[3] as f32 / 255.0;

    let out_a = sa + da * (1.0 - sa);

    if out_a <= 0.0 {
        *dst = Rgba([0, 0, 0, 0]);
        return;
    }

    let r = (src[0] as f32 * sa + dst[0] as f32 * da * (1.0 - sa)) / out_a;

    let g = (src[1] as f32 * sa + dst[1] as f32 * da * (1.0 - sa)) / out_a;

    let b = (src[2] as f32 * sa + dst[2] as f32 * da * (1.0 - sa)) / out_a;

    *dst = Rgba([
        r.round() as u8,
        g.round() as u8,
        b.round() as u8,
        (out_a * 255.0).round() as u8,
    ]);
}
