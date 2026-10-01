use iced::{Color, Point, Rectangle, advanced::image::Handle};
use image::{ImageBuffer, Rgba};

use crate::domain::{
    HexCoord, RenderTarget, assets::AssetStore, id::ImageId, layer::image::EDITOR_HEX_SIZE,
};

pub struct PngRenderTarget<'a> {
    image: &'a mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    bounds: Rectangle,
    assets: &'a AssetStore,
    hex_size: f32,
}

impl<'a> PngRenderTarget<'a> {
    pub fn new(
        image: &'a mut ImageBuffer<Rgba<u8>, Vec<u8>>,
        bounds: Rectangle,
        assets: &'a AssetStore,
        hex_size: f32,
    ) -> Self {
        Self {
            image,
            bounds,
            assets,
            hex_size,
        }
    }
}

impl RenderTarget for PngRenderTarget<'_> {
    fn hex_to_point(&self, coord: &HexCoord) -> Point {
        let point = coord.to_cartesian();

        Point::new(point.x * self.hex_size, point.y * self.hex_size)
    }

    fn get_bounds(&self) -> Rectangle {
        self.bounds * (1.0 / self.hex_size)
    }

    fn fill_polygon(&mut self, point: &Point, fill: Color) {
        let centre = Point::new(point.x - self.bounds.x, point.y - self.bounds.y);

        let vertices = hex_vertices_f(centre.x, centre.y, self.hex_size);

        fill_polygon(self.image, &vertices, fill.into_rgba8());
    }

    // Strokes are drawn with minimum thickness when exporting
    fn stroke_polygon(&mut self, point: &Point, colour: Color, _stroke_width: f32) {
        let centre = Point::new(point.x - self.bounds.x, point.y - self.bounds.y);

        let vertices = hex_vertices_f(centre.x, centre.y, self.hex_size);

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

        let scale = self.hex_size / EDITOR_HEX_SIZE;

        let x = (bounds.x * scale - self.bounds.x).round() as i64;
        let y = (bounds.y * scale - self.bounds.y).round() as i64;

        let dst_width = (bounds.width * scale).max(0.0).round() as u32;
        let dst_height = (bounds.height * scale).max(0.0).round() as u32;

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

fn hex_vertices_f(cx: f32, cy: f32, hex_size: f32) -> [(f32, f32); 6] {
    std::array::from_fn(|i| {
        let angle_rad = (60.0 * i as f32).to_radians();

        (
            cx + hex_size * angle_rad.cos(),
            cy + hex_size * angle_rad.sin(),
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

    let mut last = None;

    // The end point is excluded: the next edge of a closed polygon starts there,
    // so a shared vertex is blended once rather than twice.
    for step in 0..steps {
        let t = step as f32 / steps as f32;
        let x = x0 + dx * t;
        let y = y0 + dy * t;

        if x >= 0.0 && y >= 0.0 && x < buf.width() as f32 && y < buf.height() as f32 {
            let pixel = (x as u32, y as u32);

            if last != Some(pixel) {
                blend(buf.get_pixel_mut(pixel.0, pixel.1), colour);
                last = Some(pixel);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Layer, LayerInner, Scene, assets::ImageAsset, layer::image::ImageLayer};
    use iced::Size;

    fn solid(width: u32, height: u32, colour: [u8; 4]) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        ImageBuffer::from_pixel(width, height, Rgba(colour))
    }

    #[test]
    fn stroking_blends_over_existing_pixels_instead_of_replacing_them() {
        let mut buf = solid(10, 10, [200, 200, 200, 255]);
        draw_line(&mut buf, (0.0, 5.0), (10.0, 5.0), [0, 0, 0, 25]);

        for x in 0..10 {
            let px = buf.get_pixel(x, 5);
            assert_eq!(px[3], 255, "an opaque pixel must stay opaque");
            assert!(px[0] < 200, "the line should darken the pixel");
        }
        assert_eq!(buf.get_pixel(0, 4).0, [200, 200, 200, 255]);
    }

    #[test]
    fn every_pixel_on_a_line_is_blended_exactly_once() {
        // Slope close to 1 with a fractional length: samples can repeat a pixel.
        let mut buf = solid(20, 20, [200, 200, 200, 255]);
        draw_line(&mut buf, (0.0, 0.0), (10.0, 10.5), [0, 0, 0, 25]);

        let touched: Vec<_> = buf
            .pixels()
            .filter(|p| p.0 != [200, 200, 200, 255])
            .map(|p| p.0)
            .collect();
        assert!(!touched.is_empty());
        assert!(
            touched.iter().all(|p| *p == touched[0]),
            "a pixel was darkened more than once: {touched:?}"
        );
    }

    #[test]
    fn closed_polygon_does_not_double_blend_its_first_vertex() {
        let mut buf = solid(40, 40, [200, 200, 200, 255]);
        let square = [(5.0, 5.0), (25.0, 5.0), (25.0, 25.0), (5.0, 25.0)];
        stroke_polygon(&mut buf, &square, [0, 0, 0, 25]);

        let reference = buf.get_pixel(15, 5).0;
        assert_eq!(buf.get_pixel(5, 5).0, reference);
        assert_eq!(buf.get_pixel(25, 5).0, reference);
    }

    #[test]
    fn exported_image_lands_where_the_editor_shows_it_at_any_scale() {
        // The editor draws an image layer at `position` in editor pixels,
        // which is `position / EDITOR_HEX_SIZE` hexes.
        let mut scene = Scene::default();
        let id = scene.assets.register_image(ImageAsset {
            encoded: Vec::new(),
            extension: "png".into(),
            data: [255, 0, 0, 255].repeat(4),
            width: 2,
            height: 2,
            name: "red".into(),
        });
        let mut layer = ImageLayer::new_with(id);
        layer.position = iced::Point::new(16.0, 16.0);
        layer.set_size_ignore_aspect_ratio(Size::new(32.0, 32.0));
        let at = scene.inner.len();
        scene.insert_layer(Layer::new("img", LayerInner::Image(layer)), at);

        // hex_size 32 is twice the editor's 16, so the image is 64 px at (32, 32),
        // and the exported bounds are padded by 2 hexes (64 px) on every side.
        let bytes = crate::infrastructure::export_png(&scene, 32.0);
        let out = image::load_from_memory(&bytes).unwrap().to_rgba8();

        let (mut min_x, mut min_y, mut max_x, mut max_y) = (u32::MAX, u32::MAX, 0, 0);
        for (x, y, p) in out.enumerate_pixels() {
            // Grid lines darken red slightly but never below this.
            if p[3] == 255 && p[0] > 150 && p[1] < 40 && p[2] < 40 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }

        // Bounds start at (32 - 64) = -32 px, so the image sits at 32 - (-32) = 64.
        assert_eq!((min_x, min_y), (64, 64));
        assert_eq!((max_x, max_y), (127, 127));
    }
}
