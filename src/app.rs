use iced::{
    Color, Element, Length, Task,
    widget::{canvas, column, row},
};

use crate::{
    export::{export_png, save_bytes_as},
    state::{HexCoord, Layer},
    view::{
        CanvasSettings, HexCanvas, LayerPanel, LayerPanelMessage, Tool, layer_panel,
        toolbar_dialogue,
    },
};

pub struct App {
    layers: Vec<Layer>,

    layer_panel: LayerPanel,
    hex_canvas: CanvasSettings,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Manage current tool
    ChangeTool(Tool),

    // Manage layers
    AddLayer,
    RemoveLayer(usize),
    EditLayerName(usize, String),
    EditLayerVisibility(usize, bool),

    // Manage tiles
    PaintTile(HexCoord),
    EraseTile(HexCoord),

    LayerPanelEvent(LayerPanelMessage),

    ExportPng,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            layers: vec![Layer::new("Layer 1", Color::from_rgba8(245, 196, 168, 0.9))],
            layer_panel: LayerPanel {
                active_layer: Some(0),
                edit_layer: None,
            },
            hex_canvas: CanvasSettings {
                tool: Tool::Pan,
                hex_size: 16.0,
            },
        };
        (app, Task::none())
    }

    pub fn title(&self) -> String {
        format!("HexMap Editor")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        #[cfg(debug_assertions)]
        println!("{message:?}");

        match message {
            Message::AddLayer => {
                let colors = [
                    Color::from_rgba8(245, 196, 168, 0.9),
                    Color::from_rgba8(168, 212, 176, 0.9),
                    Color::from_rgba8(168, 200, 245, 0.9),
                    Color::from_rgba8(196, 168, 245, 0.9),
                    Color::from_rgba8(245, 168, 200, 0.9),
                ];

                let layer_count = self.layers.len();
                let new_layer = Layer::new(
                    format!("Layer {}", layer_count + 1),
                    colors[layer_count % 5],
                );
                self.layers.push(new_layer);

                return Task::done(LayerPanelMessage::SelectLayer(Some(layer_count)).into());
            }
            Message::RemoveLayer(index) => {
                self.layers.remove(index);
            }

            Message::EditLayerVisibility(index, new_state) => {
                if let Some(layer) = self.layers.get_mut(index) {
                    layer.visible = new_state;
                }
            }
            Message::EditLayerName(index, new_name) => {
                if let Some(layer) = self.layers.get_mut(index) {
                    layer.name = new_name;
                }
            }

            Message::PaintTile(hex_coord) => {
                if let Some(index) = self.layer_panel.active_layer {
                    if let Some(layer) = self.layers.get_mut(index) {
                        layer.tiles.insert(hex_coord);
                    }
                }
            }

            Message::EraseTile(hex_coord) => {
                if let Some(index) = self.layer_panel.active_layer {
                    if let Some(layer) = self.layers.get_mut(index) {
                        layer.tiles.remove(&hex_coord);
                    }
                }
            }

            Message::LayerPanelEvent(layer_panel_message) => {
                return self.layer_panel.update(layer_panel_message);
            }

            Message::ChangeTool(new_tool) => self.hex_canvas.tool = new_tool,

            Message::ExportPng => {
                let bytes = export_png(&self.layers, 64.0, 80.0);
                save_bytes_as(&bytes, "hexmap.png", "image/png");
            }
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let layer_panel = layer_panel(&self.layer_panel, &self.layers);
        let toolbar = toolbar_dialogue(&self.hex_canvas);

        let hex_canvas = HexCanvas {
            layers: &self.layers,
            settings: &self.hex_canvas,
        };
        let canvas_widget = canvas(hex_canvas).width(Length::Fill).height(Length::Fill);

        let dialogues = column![toolbar, layer_panel].width(220);

        row![dialogues, canvas_widget]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
