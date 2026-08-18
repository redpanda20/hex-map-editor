use iced::{
    Element, Subscription, Task, Theme,
    widget::{container, stack},
};

use crate::{
    domain::{History, HistoryCommand, Scene, SceneMessage},
    infrastructure::{
        IoProcess, SceneV1, export_png, load_project_async, save_bytes_async, save_project_async,
    },
    ui::{
        Inspector, InspectorMessage, Layers, LayersMessage, Panes, PanesMessage, ToastMessage,
        Toasts, Toolbar, ToolbarMessage, canvas_panel,
    },
};

pub struct App {
    pub scene: Scene,
    pub history: History,

    pub toolbar: Toolbar,
    pub layers: Layers,
    pub inspector: Inspector,
    pub toasts: Toasts,

    pub panes: Panes,
}

#[derive(Debug, Clone)]
pub enum Message {
    Panes(PanesMessage),
    Toasts(ToastMessage),
    Inspector(InspectorMessage),
    Layers(LayersMessage),
    Toolbar(ToolbarMessage),

    Scene(SceneMessage),
    History(HistoryCommand),

    Load(IoProcess<SceneV1>),
    Save(IoProcess<()>),
    Export(IoProcess<()>),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            scene: Scene::new(),
            history: History::new(),
            toolbar: Toolbar::new(),
            layers: Layers::new(),
            inspector: Inspector::new(),
            toasts: Toasts::new(),
            panes: Panes::new(),
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
            Message::Panes(message) => self.panes.update(message),
            Message::Toasts(message) => return self.toasts.update(message),
            Message::Inspector(message) => return self.inspector.update(message),
            Message::Layers(message) => return self.layers.update(message),
            Message::Toolbar(message) => return self.toolbar.update(message),

            Message::Scene(message) => {
                let edit = self.scene.update(message);
                self.history.apply(edit);
            }
            Message::History(command) => return self.history.update(command, &mut self.scene),

            Message::Export(IoProcess::Start) => {
                let bytes = export_png(&self.scene);
                return save_bytes_async(bytes, "hexmap.png");
            }
            Message::Export(IoProcess::Cancelled) => {
                eprintln!("Export cancelled");
            }
            Message::Export(IoProcess::Finished(result)) => match result {
                Ok(_) => eprintln!("Export succeeded"),
                Err(err) => eprintln!("Export failed: {err}"),
            },

            Message::Save(IoProcess::Start) => return save_project_async(&self.scene),
            Message::Save(IoProcess::Cancelled) => {
                eprintln!("Project save cancelled");
            }
            Message::Save(IoProcess::Finished(result)) => match result {
                Ok(_) => eprintln!("Project save succeeded"),
                Err(err) => eprintln!("Project save failed: {err}"),
            },

            Message::Load(IoProcess::Start) => return load_project_async(),
            Message::Load(IoProcess::Cancelled) => {
                eprintln!("Project load cancelled");
            }
            Message::Load(IoProcess::Finished(result)) => match result {
                Ok(document) => {
                    self.scene = Scene::from(document);
                }
                Err(err) => eprintln!("Project save failed: {err}"),
            },
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let inspector = |scene| self.inspector.view(scene);
        let layers = |scene| self.layers.view(scene);
        let toolbar = |scene| self.toolbar.view(scene, &self.history);

        let grid = self
            .panes
            .view(&self.scene, canvas_panel, inspector, layers, toolbar);

        let toasts = self.toasts.view().map(Message::Toasts);

        container(stack!(grid, toasts))
            .padding(2)
            .style(|theme| container::background(theme.extended_palette().background.base.color))
            .into()
    }
}
