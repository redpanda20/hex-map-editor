use std::collections::HashSet;

use iced::{Color, Task};

use crate::{app::Message, state::HexCoord};

const DEFAULT_COLORS: [Color; 5] = [
    Color::from_rgba8(245, 196, 168, 0.9),
    Color::from_rgba8(168, 212, 176, 0.9),
    Color::from_rgba8(168, 200, 245, 0.9),
    Color::from_rgba8(196, 168, 245, 0.9),
    Color::from_rgba8(245, 168, 200, 0.9),
];

pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub color: Color,

    pub tiles: HashSet<HexCoord>,
}

pub struct Layers {
    pub active_layer: Option<usize>,

    pub inner: Vec<Layer>,
}

#[derive(Debug, Clone)]
pub enum LayerMessage {
    AddLayer,
    RemoveLayer(usize),
    ChangeActiveLayer(Option<usize>),

    ChangeLayerName(usize, String),
    ChangeLayerVisibility(usize, bool),
    ChangeActiveLayerColor(Color),

    PaintTile(HexCoord),
    EraseTile(HexCoord),
}

impl Layer {
    pub fn new(name: impl Into<String>, color: Color) -> Layer {
        let name = name.into();
        let tiles = HashSet::new();
        Self {
            name,
            visible: true,
            tiles,
            color,
        }
    }
}

impl Default for Layers {
    fn default() -> Self {
        let first_layer = Layer {
            name: "Layer 1".to_string(),
            visible: true,
            color: DEFAULT_COLORS[0],
            tiles: HashSet::new(),
        };
        Self {
            active_layer: Some(0),
            inner: vec![first_layer],
        }
    }
}

impl Layers {
    pub fn get_active_layer(&self) -> Option<&Layer> {
        self.active_layer.and_then(|index| self.inner.get(index))
    }

    pub fn tiles_at_coord(&self, hex_coord: HexCoord) -> Vec<Color> {
        self.inner
            .iter()
            .filter(|layer| layer.tiles.contains(&hex_coord))
            .map(|layer| layer.color)
            .collect()
    }

    pub fn update(&mut self, message: LayerMessage) -> Task<Message> {
        let active_layer_mut = self
            .active_layer
            .and_then(|index| self.inner.get_mut(index));

        match message {
            LayerMessage::AddLayer => {
                let current_len = self.inner.len();
                let name = format!("Layer {}", current_len + 1);
                let color = DEFAULT_COLORS[current_len % 5];
                self.inner.push(Layer::new(name, color));
            }
            LayerMessage::RemoveLayer(index) => {
                self.inner.remove(index);
            }

            LayerMessage::ChangeActiveLayer(opt_index) => self.active_layer = opt_index,

            LayerMessage::ChangeLayerName(index, new_name) => {
                if let Some(layer) = self.inner.get_mut(index) {
                    layer.name = new_name;
                }
            }
            LayerMessage::ChangeLayerVisibility(index, new_visibility) => {
                if let Some(layer) = self.inner.get_mut(index) {
                    layer.visible = new_visibility;
                }
            }

            LayerMessage::ChangeActiveLayerColor(color) => {
                if let Some(layer) = active_layer_mut {
                    layer.color = color;
                };
            }

            LayerMessage::PaintTile(hex_coord) => {
                if let Some(layer) = active_layer_mut {
                    layer.tiles.insert(hex_coord);
                };
            }
            LayerMessage::EraseTile(hex_coord) => {
                if let Some(layer) = active_layer_mut {
                    layer.tiles.remove(&hex_coord);
                };
            }
        };

        Task::none()
    }
}

impl From<LayerMessage> for Message {
    fn from(value: LayerMessage) -> Self {
        Message::LayerEvent(value)
    }
}
