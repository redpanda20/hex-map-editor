use serde::{Deserialize, Serialize};

use crate::domain::layer::noise::{NoiseParams, PerlinNoiseLayer};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PerlinV1 {
    pub seed: u64,
    pub threshold: f32,
    pub frequency: f32,
    pub octaves: usize,
    pub persistence: f32,
}

impl From<&PerlinNoiseLayer> for PerlinV1 {
    fn from(perlin: &PerlinNoiseLayer) -> Self {
        let NoiseParams {
            threshold,
            frequency,
            octaves,
            persistence,
        } = perlin.get_params();

        PerlinV1 {
            seed: perlin.get_seed(),
            threshold,
            frequency,
            octaves,
            persistence,
        }
    }
}

impl From<PerlinV1> for PerlinNoiseLayer {
    fn from(wire: PerlinV1) -> Self {
        // PerlinNoiseLayer::new(seed) is deterministic
        let mut perlin = PerlinNoiseLayer::new(wire.seed);
        perlin.set_params(&NoiseParams {
            threshold: wire.threshold,
            frequency: wire.frequency,
            octaves: wire.octaves,
            persistence: wire.persistence,
        });
        perlin
    }
}
