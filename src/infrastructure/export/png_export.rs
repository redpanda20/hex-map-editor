use iced::{Color, Point, Rectangle};
use image::{ImageBuffer, Rgba};

use crate::domain::{HexCoord, RenderTarget};

use super::EXPORT_HEX_SIZE;

pub struct PngRenderTarget<'a> {
    image: &'a mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    bounds: Rectangle,
}

impl<'a> PngRenderTarget<'a> {
    pub fn new(image: &'a mut ImageBuffer<Rgba<u8>, Vec<u8>>, bounds: Rectangle) -> Self {
        Self { image, bounds }
    }
}

impl RenderTarget for PngRenderTarget<'_> {
    fn hex_to_point(&self, coord: &HexCoord) -> Point {
        let point = coord.to_cartesian();

        Point::new(point.x * EXPORT_HEX_SIZE, point.y * EXPORT_HEX_SIZE)
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds
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
