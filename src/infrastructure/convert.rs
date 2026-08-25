use std::collections::HashSet;

use iced::Color;

use crate::domain::{
    Layer, LayerInner, Scene,
    id::LayerId,
    layer::{noise::PerlinNoiseLayer, tiles::SparseTiles},
};

use super::schema::{ColourV1, HexCoordV1, LayerKindV1, LayerV1, NoiseOctavesV1, SceneV1};
use crate::domain::HexCoord;

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

impl From<HexCoord> for HexCoordV1 {
    fn from(coord: HexCoord) -> Self {
        HexCoordV1 {
            col: coord.col,
            row: coord.row,
        }
    }
}

impl From<HexCoordV1> for HexCoord {
    fn from(coord: HexCoordV1) -> Self {
        HexCoord {
            col: coord.col,
            row: coord.row,
        }
    }
}

impl From<Color> for ColourV1 {
    fn from(colour: Color) -> Self {
        ColourV1 {
            r: colour.r,
            g: colour.g,
            b: colour.b,
            a: colour.a,
        }
    }
}

impl From<ColourV1> for Color {
    fn from(colour: ColourV1) -> Self {
        Color {
            r: colour.r,
            g: colour.g,
            b: colour.b,
            a: colour.a,
        }
    }
}

// impl From<&NoiseOctaves> for NoiseOctavesV1 {
//     fn from(octaves: &NoiseOctaves) -> Self {
//         match octaves {
//             NoiseOctaves::One => NoiseOctavesV1::One,
//             NoiseOctaves::Many { count, persistence } => NoiseOctavesV1::Many {
//                 count: *count,
//                 persistence: *persistence,
//             },
//         }
//     }
// }

// ---------------------------------------------------------------------------
// Layers
// ---------------------------------------------------------------------------

fn tiles_to_v1(tiles: &HashSet<HexCoord>) -> Vec<HexCoordV1> {
    tiles.iter().copied().map(Into::into).collect()
}

fn tiles_from_v1(tiles: Vec<HexCoordV1>) -> HashSet<HexCoord> {
    tiles.into_iter().map(Into::into).collect()
}

impl From<&LayerInner> for LayerKindV1 {
    fn from(inner: &LayerInner) -> Self {
        match inner {
            LayerInner::Tiles(store) if store.is_inverted() => LayerKindV1::InvertedTiles {
                colour: store.colour.into(),
                tiles: tiles_to_v1(store.get_all_tiles()),
            },
            LayerInner::Tiles(store) => LayerKindV1::Tiles {
                colour: store.colour.into(),
                tiles: tiles_to_v1(store.get_all_tiles()),
            },
            LayerInner::Perlin(perlin) => LayerKindV1::Perlin {
                seed: perlin.get_seed(),
                threshold: perlin.threshold,
                frequency: perlin.frequency,
                octaves: match perlin.octaves {
                    1 => NoiseOctavesV1::One,
                    count => NoiseOctavesV1::Many {
                        count,
                        persistence: perlin.persistence,
                    },
                },
            },
        }
    }
}

impl From<LayerKindV1> for LayerInner {
    fn from(kind: LayerKindV1) -> Self {
        match kind {
            LayerKindV1::Tiles { colour, tiles } => {
                LayerInner::Tiles(SparseTiles::new_with(colour.into(), tiles_from_v1(tiles)))
            }
            LayerKindV1::InvertedTiles { colour, tiles } => {
                let mut store = SparseTiles::new_with(colour.into(), tiles_from_v1(tiles));
                store.invert();
                LayerInner::Tiles(store)
            }
            LayerKindV1::Perlin {
                seed,
                threshold,
                frequency,
                octaves,
            } => {
                // `PerlinNoiseLayer::new` deterministically derives its gradient
                // table from the seed, so reconstructing via `new` + setters
                // exactly reproduces the original layer.
                let mut perlin = PerlinNoiseLayer::new(seed);
                perlin.threshold = threshold;
                perlin.frequency = frequency;
                let (octaves, persistence) = match octaves {
                    NoiseOctavesV1::One => (1, 0.5),
                    NoiseOctavesV1::Many { count, persistence } => (count, persistence),
                };
                perlin.octaves = octaves;
                perlin.persistence = persistence;
                LayerInner::Perlin(perlin)
            }
        }
    }
}

impl From<&Layer> for LayerV1 {
    fn from(layer: &Layer) -> Self {
        LayerV1 {
            name: layer.name.clone(),
            visible: layer.visible,
            kind: (&layer.kind).into(),
        }
    }
}

impl From<LayerV1> for Layer {
    fn from(layer: LayerV1) -> Self {
        Layer {
            id: LayerId::next(),
            name: layer.name,
            visible: layer.visible,
            kind: layer.kind.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Document <-> Layers
// ---------------------------------------------------------------------------

impl From<&Scene> for SceneV1 {
    fn from(layers: &Scene) -> Self {
        SceneV1 {
            layers: layers.inner.iter().map(Into::into).collect(),
            active_layer: None,
        }
    }
}

impl From<SceneV1> for Scene {
    fn from(document: SceneV1) -> Self {
        let inner = document.layers.into_iter().map(Into::into).collect();
        Scene::from_layers(inner)
    }
}
