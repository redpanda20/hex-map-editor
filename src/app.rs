use iced::{
    Color, Element, Length, Task, Theme,
    widget::{canvas, container, pane_grid},
};

use crate::{
    export::{export_png, save_bytes_as},
    state::{HexCoord, Layer, Tool},
    view::{
        HexCanvas, LayerPanel, LayerPanelMessage, PaneType, default_pane_config, layer_panel,
        toolbar_panel,
    },
};

pub struct App {
    layers: Vec<Layer>,
    active_tool: Tool,

    panes: pane_grid::State<PaneType>,
    layer_panel: LayerPanel,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Manage layers
    AddLayer,
    RemoveLayer(usize),
    EditLayerName(usize, String),
    EditLayerVisibility(usize, bool),

    // Manage current tool
    ChangeTool(Tool),

    // Manage tiles
    PaintTile(HexCoord),
    EraseTile(HexCoord),

    // Sub element
    LayerPanelEvent(LayerPanelMessage),

    // Manage panels
    PaneResized(pane_grid::ResizeEvent),

    ExportPng,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let layers = vec![Layer::new("Layer 1", Color::from_rgba8(245, 196, 168, 0.9))];

        let panes = pane_grid::State::with_configuration(default_pane_config());

        let app = Self {
            layers,
            layer_panel: LayerPanel::new(),
            panes,
            active_tool: Tool::default(),
        };
        (app, Task::none())
    }

    pub fn title(&self) -> String {
        format!("HexMap Editor")
    }

    pub fn theme(&self) -> Option<Theme> {
        Some(Theme::Dark)
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

            Message::ChangeTool(new_tool) => self.active_tool = new_tool,

            Message::ExportPng => {
                let bytes = export_png(&self.layers, 64.0, 80.0);
                save_bytes_as(&bytes, "hexmap.png", "image/png");
            }
            Message::PaneResized(resize_event) => {
                let pane_grid::ResizeEvent { split, ratio } = resize_event;
                self.panes.resize(split, ratio);
            }
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let grid = pane_grid(&self.panes, |_id, state, _is_maximised| {
            let inner: Element<'_, Message> = match state {
                PaneType::Toolbar => container(toolbar_panel(&self.active_tool))
                    .style(container::bordered_box)
                    .into(),
                PaneType::Canvas => {
                    let hex_canvas = HexCanvas {
                        layers: &self.layers,
                        tool: &self.active_tool,
                    };
                    canvas(hex_canvas)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                }
                PaneType::Layers => container(layer_panel(&self.layer_panel, &self.layers))
                    .style(container::bordered_box)
                    .into(),
            };

            pane_grid::Content::new(inner)
        })
        .on_resize(10, Message::PaneResized)
        .spacing(2);

        container(grid)
            .padding(2)
            .style(|theme| container::background(theme.extended_palette().background.base.color))
            .into()
    }
}
