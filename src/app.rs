use iced::{
    Element, Subscription, Task, Theme,
    widget::{container, pane_grid, stack},
};

use crate::{
    export::{export_png, save_bytes_async},
    panels::{
        PaneType, ToastEvent, Toasts, canvas_panel, default_pane_config, inspector_panel,
        layer_stack_panel, toast_widget, toolbar_panel,
    },
    persistence::{Document, load_project_async, save_project_async},
    state::{LayerMessage, LayerType, Layers, Tool},
};

#[derive(Default, Debug)]
pub struct EditorState {
    pub active_layer_name: Option<String>,
    pub active_layer_type: LayerType,
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
    ChangeLayerType(LayerType),

    PaneResized(pane_grid::ResizeEvent),

    ExportPng,
    ExportCancelled,
    Exported(Result<(), String>),

    SaveProject,
    ProjectSaveCancelled,
    ProjectSaved(Result<(), String>),

    LoadProject,
    ProjectLoadCancelled,
    ProjectLoaded(Result<Document, String>),
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
                let bytes = export_png(&self.layers);
                return save_bytes_async(bytes, "hexmap.png");
            }
            Message::ExportCancelled => {
                eprintln!("Export cancelled");
            }
            Message::Exported(result) => match result {
                Ok(_) => eprintln!("Export succeeded"),
                Err(err) => eprintln!("Export failed: {err}"),
            },

            Message::SaveProject => return save_project_async(&self.layers),
            Message::ProjectSaveCancelled => {
                eprintln!("Project save cancelled");
            }
            Message::ProjectSaved(result) => match result {
                Ok(_) => eprintln!("Save succeeded"),
                Err(err) => eprintln!("Save failed: {err}"),
            },

            Message::LoadProject => return load_project_async(),
            Message::ProjectLoadCancelled => {
                eprintln!("Project load cancelled");
            }
            Message::ProjectLoaded(result) => match result {
                Ok(document) => {
                    self.layers = Layers::from(document);
                    self.editor_state = EditorState::default();
                }
                Err(err) => eprintln!("Load failed: {err}"),
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
            Message::ChangeLayerType(layer_type) => {
                self.editor_state.active_layer_type = layer_type
            }
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let grid = pane_grid(&self.panes, |_id, state, _is_maximised| {
            let inner: Element<'_, Message> = match state {
                PaneType::Toolbar => toolbar_panel(&self.active_tool),
                PaneType::LayerStack => layer_stack_panel(&self.layers, &self.editor_state),
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
