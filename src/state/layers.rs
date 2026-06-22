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
    active_layer: Option<usize>,

    inner: Vec<Layer>,
}

#[derive(Debug, Clone)]
pub enum LayerMessage {
    AddLayer,
    RemoveLayer(usize),
    ChangeActiveLayer(Option<usize>),

    ChangeLayerName(usize, String),
    ChangeLayerVisibility(usize, bool),

    ChangeActiveLayerVisibility,
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
        let layer = Layer::new("Layer 1", DEFAULT_COLORS[0]);
        Layers {
            active_layer: Some(0),
            inner: vec![layer],
        }
    }
}

impl Layers {
    fn canonicalize_name(&self, base_name: &str) -> String {
        let mut name = format!("{base_name} 1");

        for count in 0..self.inner.len() + 1 {
            let search = self.inner.iter().filter(|layer| layer.name == name).last();
            if search.is_none() {
                break;
            } else {
                name = format!("{base_name} {}", count + 1);
            }
        }

        name
    }

    pub fn get_active_layer(&self) -> Option<&Layer> {
        self.active_layer.and_then(|index| self.inner.get(index))
    }

    pub fn get_layers(&self) -> &[Layer] {
        &self.inner
    }

    pub fn is_active_layer(&self, index: usize) -> bool {
        self.active_layer == Some(index)
    }

    pub fn tiles_at_coord(&self, hex_coord: HexCoord) -> Vec<Color> {
        self.inner
            .iter()
            .filter(|layer| layer.visible)
            .filter(|layer| layer.tiles.contains(&hex_coord))
            .map(|layer| layer.color)
            .collect()
    }

    fn add_layer(&mut self, name: &str) {
        let name = self.canonicalize_name(&name);
        let color = DEFAULT_COLORS[self.inner.len() % 5];
        self.inner.push(Layer::new(name, color));
    }

    pub fn update(&mut self, message: LayerMessage) -> Task<Message> {
        let active_layer_mut = self
            .active_layer
            .and_then(|index| self.inner.get_mut(index));

        match message {
            LayerMessage::AddLayer => {
                self.add_layer("Layer");
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
            LayerMessage::ChangeActiveLayerVisibility => {
                if let Some(layer) = active_layer_mut {
                    layer.visible = !layer.visible;
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
