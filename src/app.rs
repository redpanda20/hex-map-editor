use iced::{
    Element, Length, Task, Theme, font,
    widget::{canvas, container, pane_grid},
};
use iced_fonts::BOOTSTRAP_FONT_BYTES;

use crate::{
    export::{export_png, save_bytes_as},
    state::{LayerMessage, Layers, Tool},
    view::{
        HexCanvas, LayerPanel, LayerPanelMessage, PaneType, colour_panel, default_pane_config,
        layer_panel, toolbar_panel,
    },
};

pub struct App {
    layers: Layers,
    active_tool: Tool,

    panes: pane_grid::State<PaneType>,
    layer_panel: LayerPanel,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Manage current tool
    ChangeTool(Tool),

    // Layers
    LayerEvent(LayerMessage),

    // Layer Panel
    LayerPanelEvent(LayerPanelMessage),

    // Panel management
    PaneResized(pane_grid::ResizeEvent),

    FontLoaded(Result<(), font::Error>),
    ExportPng,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let panes = pane_grid::State::with_configuration(default_pane_config());

        let app = Self {
            layers: Layers::default(),
            layer_panel: LayerPanel::new(),
            panes,
            active_tool: Tool::default(),
        };

        let font_load_task = font::load(BOOTSTRAP_FONT_BYTES).map(Message::FontLoaded);

        (app, Task::batch(vec![font_load_task]))
    }

    pub fn title(&self) -> String {
        format!("HexMap Editor")
    }

    pub fn theme(&self) -> Option<Theme> {
        None
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        #[cfg(debug_assertions)]
        println!("{message:?}");

        match message {
            Message::LayerEvent(layers_message) => {
                return self.layers.update(layers_message);
            }

            Message::LayerPanelEvent(layer_panel_message) => {
                return self.layer_panel.update(layer_panel_message);
            }

            Message::ChangeTool(new_tool) => self.active_tool = new_tool,

            Message::ExportPng => {
                let bytes = export_png(&self.layers.inner);
                save_bytes_as(&bytes, "hexmap.png", "image/png");
            }
            Message::PaneResized(resize_event) => {
                let pane_grid::ResizeEvent { split, ratio } = resize_event;
                self.panes.resize(split, ratio);
            }
            Message::FontLoaded(result) => match result {
                Ok(_) => eprintln!("Font loaded"),
                Err(err) => eprintln!("Font failed to load: {err:?}"),
            },
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let grid = pane_grid(&self.panes, |_id, state, _is_maximised| {
            let inner: Element<'_, Message> = match state {
                PaneType::Toolbar => toolbar_panel(&self.active_tool),
                PaneType::Layers => layer_panel(&self.layer_panel, &self.layers),
                PaneType::Colour => colour_panel(&self.layers),

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
