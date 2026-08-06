mod noise;
mod tile_store;

use iced::Color;
pub use tile_store::SparseTiles;

use crate::state::{HexCoord, flood_fill};

const DEFAULT_COLORS: [Color; 5] = [
    Color::from_rgba8(245, 196, 168, 0.9),
    Color::from_rgba8(168, 212, 176, 0.9),
    Color::from_rgba8(168, 200, 245, 0.9),
    Color::from_rgba8(196, 168, 245, 0.9),
    Color::from_rgba8(245, 168, 200, 0.9),
];

pub enum LayerInner {
    Tiles(SparseTiles),
    InvertedTiles(SparseTiles),
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
    AddDefaultLayer(String),
    RemoveLayer(usize),
    SwapLayers(usize, usize),

    SetActiveLayer(Option<usize>),

    EditLayerVisibility(usize, bool),
    EditLayerName(usize, String),
    EditLayerColour(usize, Color),

    PaintHex(HexCoord),
    EraseHex(HexCoord),
    FillFromHex(HexCoord),
}

pub struct Layers {
    pub inner: Vec<Layer>,
    pub active_layer: Option<usize>,
}

impl Default for Layers {
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
        }
    }
}

impl Layers {
    /// TODO: Check which names are in use and pick the smallest number
    fn canonacalize_name(&self, name: String) -> String {
        let layer_count = self.inner.len() + 1;
        format!("{name} {layer_count}")
    }

    pub fn update(&mut self, message: LayerMessage) {
        match message {
            // --- Edit Layers ---
            LayerMessage::AddDefaultLayer(name) => {
                let colour = DEFAULT_COLORS[self.inner.len() % 5];
                let new_layer = Layer {
                    name: self.canonacalize_name(name),
                    visible: true,
                    inner: LayerInner::Tiles(SparseTiles::new(colour)),
                };
                self.inner.push(new_layer);
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
            LayerMessage::EditLayerColour(index, new_colour) => {
                if let Some(layer) = self.inner.get_mut(index) {
                    match &mut layer.inner {
                        LayerInner::Tiles(sparse_tiles) => sparse_tiles.set_colour(new_colour),
                        LayerInner::InvertedTiles(sparse_tiles) => {
                            sparse_tiles.set_colour(new_colour)
                        }
                    }
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
                    };

                    // If the layer is empty, short circuit and invert
                    let Some(bounds) = store.get_bounds() else {
                        layer.inner = match &layer.inner {
                            LayerInner::Tiles(store) => LayerInner::InvertedTiles(store.clone()),
                            LayerInner::InvertedTiles(store) => LayerInner::Tiles(store.clone()),
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
                        },
                        None => {
                            layer.inner = match &layer.inner {
                                LayerInner::Tiles(store) => {
                                    LayerInner::InvertedTiles(store.clone())
                                }
                                LayerInner::InvertedTiles(store) => {
                                    LayerInner::Tiles(store.clone())
                                }
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
