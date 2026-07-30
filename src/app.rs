use iced::{
    Element, Subscription, Task, Theme,
    widget::{container, pane_grid, stack},
};

use crate::{
    // export::{export_png, save_bytes_async},
    panels::{
        LayerEvent, LayerManager, PaneType, ToastEvent, Toasts, canvas_panel, default_pane_config,
        layer_details, layer_panel, toast_widget, toolbar_panel,
    },
    state::{LayerMessage, Layers, Tool},
};

pub struct App {
    layers: Layers,
    active_tool: Tool,

    panes: pane_grid::State<PaneType>,
    layer_panel: LayerManager,
    toasts: Toasts,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Manage current tool
    ChangeTool(Tool),

    // Layers
    LayerEvent(LayerMessage),

    // Layer Panel
    LayerPanelEvent(LayerEvent),
    ToastEvent(ToastEvent),

    // Panel management
    PaneResized(pane_grid::ResizeEvent),

    ExportPng,
    ExportCancelled,
    Exported(Result<(), String>),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let panes = pane_grid::State::with_configuration(default_pane_config());

        let app = Self {
            layers: Layers::default(),
            toasts: Toasts::new(),
            layer_panel: LayerManager::new(),
            panes,
            active_tool: Tool::default(),
        };

        (app, Task::none())
    }

    pub fn title(&self) -> String {
        format!("HexMap Editor")
    }

    pub fn theme(&self) -> Option<Theme> {
        None
    }

    pub fn subscription(&self) -> Subscription<Message> {
        self.toasts.subscription()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        #[cfg(debug_assertions)]
        println!("{message:?}");

        self.toasts.listen_to_events(&message);

        match message {
            Message::LayerEvent(layers_message) => {
                self.layers.update(layers_message);
            }

            Message::LayerPanelEvent(layer_panel_message) => {
                return self.layer_panel.update(layer_panel_message);
            }

            Message::ToastEvent(toast_event) => {
                return self.toasts.update(toast_event);
            }

            Message::ChangeTool(new_tool) => self.active_tool = new_tool,

            Message::ExportPng => {
                todo!()
                // let bytes = export_png(&self.layers.get_layers());
                // return save_bytes_async(bytes, "hexmap.png");
                // save_bytes_as(bytes, "hexmap.png", "image/png");
            }
            Message::ExportCancelled => {}
            Message::Exported(result) => match result {
                Ok(_) => eprintln!("Export succeeded"),
                Err(err) => eprintln!("Export failed: {err}"),
            },
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
                PaneType::Toolbar => toolbar_panel(&self.active_tool),
                PaneType::Layers => layer_panel(&self.layer_panel, &self.layers),
                PaneType::Colour => layer_details(&self.layers),
                PaneType::Canvas => canvas_panel(&self.layers, &self.active_tool),
            };

            pane_grid::Content::new(inner)
        })
        .on_resize(10, Message::PaneResized)
        .spacing(2);

        container(stack!(grid, toast_widget(&self.toasts)))
            .padding(2)
            .style(|theme| container::background(theme.extended_palette().background.base.color))
            .into()
    }
}
