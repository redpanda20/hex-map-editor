pub mod noise;
mod tile_store;

use iced::Color;
pub use tile_store::SparseTiles;

use crate::state::{HexCoord, PerlinNoiseLayer, flood_fill};

const DEFAULT_COLORS: [Color; 5] = [
    Color::from_rgba8(245, 196, 168, 0.9),
    Color::from_rgba8(168, 212, 176, 0.9),
    Color::from_rgba8(168, 200, 245, 0.9),
    Color::from_rgba8(196, 168, 245, 0.9),
    Color::from_rgba8(245, 168, 200, 0.9),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerType {
    Tiles,
    PerlinNoise,
}
impl Default for LayerType {
    fn default() -> Self {
        LayerType::Tiles
    }
}
impl ToString for LayerType {
    fn to_string(&self) -> String {
        match self {
            LayerType::Tiles => "Tile".to_string(),
            LayerType::PerlinNoise => "Noise".to_string(),
        }
    }
}

pub enum LayerInner {
    Tiles(SparseTiles),
    InvertedTiles(SparseTiles),
    Perlin(noise::PerlinNoiseLayer),
}

impl LayerInner {
    pub fn draw<T>(
        &self,
        target: &mut T,
        coords: impl Iterator<Item = HexCoord>,
        mut draw: impl FnMut(&mut T, HexCoord, Color),
    ) {
        for coord in coords {
            match self {
                LayerInner::Tiles(sparse_tiles) => {
                    if sparse_tiles.exists_at(&coord) {
                        draw(target, coord, sparse_tiles.colour_at(&coord))
                    }
                }
                LayerInner::InvertedTiles(sparse_tiles) => {
                    if !sparse_tiles.exists_at(&coord) {
                        draw(target, coord, sparse_tiles.colour_at(&coord))
                    }
                }
                LayerInner::Perlin(perlin_noise_layer) => {
                    if perlin_noise_layer.exists_at(&coord) {
                        draw(target, coord, perlin_noise_layer.colour_at(&coord))
                    }
                }
            }
        }
    }
}

pub struct Layer {
    pub name: String,
    pub visible: bool,

    pub inner: LayerInner,
}

#[derive(Debug, Clone)]
pub enum LayerMessage {
    AddLayer(String, LayerType),
    RemoveLayer(usize),
    SwapLayers(usize, usize),

    SetActiveLayer(Option<usize>),

    EditLayerVisibility(usize, bool),
    EditLayerName(usize, String),
    EditLayerFistColour(usize, Color),
    EditLayerSeed(usize, u64),
    EditLayerScale(usize, f32),
    EditLayerThreshold(usize, f32),
    EditLayerOctaves(usize, usize),
    EditLayerPersistence(usize, f32),

    PaintHex(HexCoord),
    EraseHex(HexCoord),
    FillFromHex(HexCoord),
}

pub struct Scene {
    pub inner: Vec<Layer>,
    pub active_layer: Option<usize>,

    revision: u64,
}

impl Default for Scene {
    fn default() -> Self {
        let inner = LayerInner::Tiles(tile_store::SparseTiles::new(DEFAULT_COLORS[0]));

        let layer = Layer {
            name: "Layer 1".to_string(),
            visible: true,
            inner,
        };

        Self {
            inner: vec![layer],
            active_layer: Some(0),
            revision: 0,
        }
    }
}

impl Scene {
    pub fn replace_layers(inner: Vec<Layer>) -> Self {
        Self {
            inner,
            active_layer: None,
            revision: 1,
        }
    }

    /// TODO: Check which names are in use and pick the smallest number
    fn canonacalize_name(&self, name: String) -> String {
        let layer_count = self.inner.len() + 1;
        format!("{name} {layer_count}")
    }

    /// A counter incremented when the state of Layers changes.
    /// Used by consumers to invalidate out of date caches
    ///
    /// TODO: Only update on meangiful content changes
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn update(&mut self, message: LayerMessage) {
        self.revision = self.revision.wrapping_add(1);

        match message {
            // --- Edit Layers ---
            LayerMessage::AddLayer(name, layer_type) => {
                let name = self.canonacalize_name(name);
                let visible = true;
                let inner = match layer_type {
                    LayerType::Tiles => {
                        let colour = DEFAULT_COLORS[self.inner.len() % 5];
                        LayerInner::Tiles(SparseTiles::new(colour))
                    }
                    LayerType::PerlinNoise => {
                        let seed = rand::random();
                        LayerInner::Perlin(PerlinNoiseLayer::new(seed))
                    }
                };

                self.inner.push(Layer {
                    name,
                    visible,
                    inner,
                });
            }

            LayerMessage::RemoveLayer(index) => {
                if index >= self.inner.len() {
                    return;
                }

                self.inner.remove(index);
            }
            LayerMessage::SwapLayers(a, b) => {
                if a >= self.inner.len() || b >= self.inner.len() {
                    return;
                }

                self.inner.swap(a, b);
            }
            LayerMessage::SetActiveLayer(some_index) => self.active_layer = some_index,

            // --- Edit layer properties ---
            LayerMessage::EditLayerVisibility(index, new_visibility) => {
                if let Some(layer) = self.inner.get_mut(index) {
                    layer.visible = new_visibility
                }
            }
            LayerMessage::EditLayerName(index, new_name) => {
                if let Some(layer) = self.inner.get_mut(index) {
                    layer.name = new_name
                }
            }
            LayerMessage::EditLayerFistColour(index, new_colour) => {
                if let Some(layer) = self.inner.get_mut(index) {
                    match &mut layer.inner {
                        LayerInner::Tiles(sparse_tiles) => sparse_tiles.set_colour(new_colour),
                        LayerInner::InvertedTiles(sparse_tiles) => {
                            sparse_tiles.set_colour(new_colour)
                        }
                        LayerInner::Perlin(_) => todo!(),
                    }
                }
            }
            // --- Edit Layer Properties (Proc gen) ---
            LayerMessage::EditLayerSeed(index, seed) => match self.inner.get_mut(index) {
                Some(Layer {
                    name: _,
                    visible: _,
                    inner: LayerInner::Perlin(perlin_noise_layer),
                }) => {
                    perlin_noise_layer.set_seed(seed);
                }
                _ => (),
            },
            LayerMessage::EditLayerScale(index, scale) => match self.inner.get_mut(index) {
                Some(Layer {
                    name: _,
                    visible: _,
                    inner: LayerInner::Perlin(perlin_noise_layer),
                }) => {
                    perlin_noise_layer.set_scale(scale);
                }
                _ => (),
            },
            LayerMessage::EditLayerThreshold(index, threshold) => match self.inner.get_mut(index) {
                Some(Layer {
                    name: _,
                    visible: _,
                    inner: LayerInner::Perlin(perlin_noise_layer),
                }) => {
                    perlin_noise_layer.set_threshold(threshold);
                }
                _ => (),
            },
            LayerMessage::EditLayerOctaves(index, count) => match self.inner.get_mut(index) {
                Some(Layer {
                    name: _,
                    visible: _,
                    inner: LayerInner::Perlin(perlin_noise_layer),
                }) => {
                    let current_persistence = match &mut perlin_noise_layer.octaves {
                        noise::NoiseOctaves::One => None,
                        noise::NoiseOctaves::Many {
                            count: _,
                            persistence,
                        } => Some(*persistence),
                    };
                    if count == 1 {
                        perlin_noise_layer.set_single_octave();
                    } else {
                        perlin_noise_layer.set_octaves(count, current_persistence.unwrap_or(0.5));
                    }
                }
                _ => (),
            },
            LayerMessage::EditLayerPersistence(index, new_persistence) => {
                match self.inner.get_mut(index) {
                    Some(Layer {
                        name: _,
                        visible: _,
                        inner: LayerInner::Perlin(perlin_noise_layer),
                    }) => match &mut perlin_noise_layer.octaves {
                        noise::NoiseOctaves::One => (),
                        noise::NoiseOctaves::Many {
                            count: _,
                            persistence,
                        } => *persistence = new_persistence,
                    },
                    _ => (),
                }
            }

            // --- Edit tiles ---
            LayerMessage::PaintHex(hex_coord) => {
                if let Some(layer) = self
                    .active_layer
                    .and_then(|index| self.inner.get_mut(index))
                {
                    match &mut layer.inner {
                        LayerInner::Tiles(store) => store.paint(hex_coord),
                        LayerInner::InvertedTiles(store) => store.erase(hex_coord),
                        // Noise layer cannot be drawn to
                        LayerInner::Perlin(_) => (),
                    }
                }
            }
            LayerMessage::EraseHex(hex_coord) => {
                if let Some(layer) = self
                    .active_layer
                    .and_then(|index| self.inner.get_mut(index))
                {
                    match &mut layer.inner {
                        LayerInner::Tiles(store) => store.erase(hex_coord),
                        LayerInner::InvertedTiles(store) => store.paint(hex_coord),
                        // Noise layer cannot be drawn changed
                        LayerInner::Perlin(_) => (),
                    }
                }
            }

            LayerMessage::FillFromHex(hex_coord) => {
                if let Some(layer) = self
                    .active_layer
                    .and_then(|index| self.inner.get_mut(index))
                {
                    let store = match &layer.inner {
                        LayerInner::Tiles(store) => store,
                        LayerInner::InvertedTiles(store) => store,
                        LayerInner::Perlin(_) => return,
                    };

                    // If the layer is empty, short circuit and invert
                    let Some(bounds) = store.get_bounds() else {
                        layer.inner = match &layer.inner {
                            LayerInner::Tiles(store) => LayerInner::InvertedTiles(store.clone()),
                            LayerInner::InvertedTiles(store) => LayerInner::Tiles(store.clone()),
                            LayerInner::Perlin(_) => return,
                        };
                        return;
                    };

                    match flood_fill(hex_coord, &store.get_all_tiles(), bounds) {
                        Some(region) => match &mut layer.inner {
                            LayerInner::Tiles(store) => {
                                region.into_iter().for_each(|c| store.paint(c))
                            }
                            LayerInner::InvertedTiles(store) => {
                                region.into_iter().for_each(|c| store.erase(c))
                            }
                            LayerInner::Perlin(_) => return,
                        },
                        None => {
                            layer.inner = match &layer.inner {
                                LayerInner::Tiles(store) => {
                                    LayerInner::InvertedTiles(store.clone())
                                }
                                LayerInner::InvertedTiles(store) => {
                                    LayerInner::Tiles(store.clone())
                                }
                                LayerInner::Perlin(_) => return,
                            };
                        }
                    }
                }
            }
        }
    }

    pub fn get_visible_layers(&self) -> Vec<&LayerInner> {
        self.inner
            .iter()
            .filter(|layer| layer.visible)
            .map(|layer| &layer.inner)
            .collect::<Vec<_>>()
    }
}
