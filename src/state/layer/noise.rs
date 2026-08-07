use std::hash::{DefaultHasher, Hash, Hasher};

use iced::{Color, Vector};
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::state::{HexBounds, HexCoord};

enum NoiseOctaves {
    One,
    Many { count: usize, persistence: f32 },
}

const TABLE_SIZE: usize = 64;

pub struct PerlinNoiseLayer {
    pub seed: u64,
    gradient_table: [Vector; TABLE_SIZE],
    pub threshold: f32,

    pub scale: f32,
    _octaves: NoiseOctaves,
}

impl PerlinNoiseLayer {
    pub fn new(seed: u64) -> PerlinNoiseLayer {
        let mut rng = SmallRng::seed_from_u64(seed);
        let gradient_table: [Vector; TABLE_SIZE] = std::array::from_fn(|_| {
            let x = rng.random::<f32>() * 2.0 - 1.0;
            let y = rng.random::<f32>() * 2.0 - 1.0;
            let size = (x * x + y * y).sqrt();
            Vector::new(x / size, y / size)
        });

        Self {
            seed,
            gradient_table,
            threshold: 1.0,
            scale: 5.0,
            _octaves: NoiseOctaves::One,
        }
    }

    /// Samples perlin noise at a given x,y coordinate
    /// Returns a float the range [-1, 1]
    fn sample(&self, x: f32, y: f32) -> f32 {
        let (x, y) = (x * self.scale, y * self.scale);

        let (x0, y0) = (x.floor(), y.floor());
        let (x1, y1) = (x0 + 1.0, y0 + 1.0);
        let (u, v) = (x - x0, y - y0);

        // Get gradient at the corners of the unit square containing (x, y)
        let grad_00 = self.direction_of(x0 as i32, y0 as i32);
        let grad_01 = self.direction_of(x0 as i32, y1 as i32);
        let grad_10 = self.direction_of(x1 as i32, y0 as i32);
        let grad_11 = self.direction_of(x1 as i32, y1 as i32);

        // Get vectors to the edges of the corners square
        let vec_00 = Vector::new(u, v);
        let vec_01 = Vector::new(u, v - 1.0);
        let vec_10 = Vector::new(u - 1.0, v);
        let vec_11 = Vector::new(u - 1.0, v - 1.0);

        // Dot product is equivalent to calculating the contribution * vector
        let contrib_00 = dot(grad_00, vec_00);
        let contrib_01 = dot(grad_01, vec_01);
        let contrib_10 = dot(grad_10, vec_10);
        let contrib_11 = dot(grad_11, vec_11);

        // Apply ease curve. Function has 0 as its 2nd and 3rd derivatives
        let smooth_u = 6.0 * u.powi(5) - 15.0 * u.powi(4) + 10.0 * u.powi(3);
        let smooth_v = 6.0 * v.powi(5) - 15.0 * v.powi(4) + 10.0 * v.powi(3);

        // Blend between contributions
        lerp(
            lerp(contrib_00, contrib_01, smooth_v),
            lerp(contrib_10, contrib_11, smooth_v),
            smooth_u,
        )
    }

    fn direction_of(&self, x: i32, y: i32) -> Vector {
        let mut hasher = DefaultHasher::new();
        x.hash(&mut hasher);
        y.hash(&mut hasher);
        let index = hasher.finish() as usize % TABLE_SIZE;
        self.gradient_table[index]
    }
}

fn lerp(lhs: f32, rhs: f32, amount: f32) -> f32 {
    lhs * (1.0 - amount) + rhs * amount
}

fn dot(lhs: Vector, rhs: Vector) -> f32 {
    lhs.x * rhs.x + lhs.y * rhs.y
}

impl PerlinNoiseLayer {
    pub fn exists_at(&self, location: &HexCoord) -> bool {
        let Vector { x, y } = location.to_cartesian();
        let sample = self.sample(x, y);
        let normalized = sample * 0.5 + 0.5;
        normalized >= self.threshold
    }

    pub fn colour_at(&self, location: &HexCoord) -> Color {
        let Vector { x, y } = location.to_cartesian();
        let sample = self.sample(x, y);
        let normalized = sample * 0.5 + 0.5;
        Color::from_rgb(normalized, normalized, normalized)
    }

    pub fn get_bounds(&self) -> Option<HexBounds> {
        None
    }
}

impl PerlinNoiseLayer {
    pub fn set_seed(&mut self, seed: u64) {
        let mut rng = SmallRng::seed_from_u64(seed);
        let gradient_table: [Vector; TABLE_SIZE] = std::array::from_fn(|_| {
            let x = rng.random::<f32>() * 2.0 - 1.0;
            let y = rng.random::<f32>() * 2.0 - 1.0;
            let size = (x * x + y * y).sqrt();
            Vector::new(x / size, y / size)
        });

        self.seed = seed;
        self.gradient_table = gradient_table
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale
    }

    /// Expects a threshold between 0 and 1 inclusive
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold
    }
}
