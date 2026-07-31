use iced::{
    Element, Subscription, Task, Theme,
    widget::{container, pane_grid, stack},
};

use crate::{
    panels::{
        PaneType, ToastEvent, Toasts, canvas_panel, default_pane_config, inspector_panel,
        layer_stack_panel, toast_widget, toolbar_panel,
    },
    state::{LayerMessage, Layers, Tool},
};

#[derive(Default, Debug)]
pub struct EditorState {
    pub active_layer_name: Option<String>,
}

pub struct App {
    layers: Layers,
    active_tool: Tool,
    editor_state: EditorState,

    panes: pane_grid::State<PaneType>,
    toasts: Toasts,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Manage current tool
    ChangeTool(Tool),

    // Layers
    LayerEvent(LayerMessage),

    ToastEvent(ToastEvent),

    // Panel management
    LayerRenameStart(String),
    LayerRename(Option<String>),
    LayerRenameSubmit(usize),

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
            editor_state: EditorState::default(),
            toasts: Toasts::new(),
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
                self.editor_state.active_layer_name = None;
                self.layers.update(layers_message);
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

            Message::LayerRename(new_name) => self.editor_state.active_layer_name = new_name,
            Message::LayerRenameStart(name) => self.editor_state.active_layer_name = Some(name),
            Message::LayerRenameSubmit(index) => {
                if let Some(new_name) = &self.editor_state.active_layer_name {
                    return Task::done(Message::LayerEvent(LayerMessage::EditLayerName(
                        index,
                        new_name.clone(),
                    )));
                }
                self.editor_state.active_layer_name = None
            }
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let grid = pane_grid(&self.panes, |_id, state, _is_maximised| {
            let inner: Element<'_, Message> = match state {
                PaneType::Toolbar => toolbar_panel(&self.active_tool),
                PaneType::LayerStack => layer_stack_panel(&self.layers),
                PaneType::Canvas => canvas_panel(&self.layers, &self.active_tool),
                PaneType::Inspector => inspector_panel(&self.layers, &self.editor_state),
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
