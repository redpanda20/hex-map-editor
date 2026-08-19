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
        Inspector, InspectorMessage, KeybindMessage, Keybinds, Layers, LayersMessage, Panes,
        PanesMessage, ToastMessage, Toasts, Toolbar, ToolbarMessage, canvas_panel,
    },
};

pub struct App {
    pub scene: Scene,
    pub history: History,

    pub toolbar: Toolbar,
    pub layers: Layers,
    pub inspector: Inspector,

    pub toasts: Toasts,
    pub keybinds: Keybinds,
    pub panes: Panes,
}

#[derive(Debug, Clone)]
pub enum Message {
    Scene(SceneMessage),
    History(HistoryCommand),

    Inspector(InspectorMessage),
    Layers(LayersMessage),
    Toolbar(ToolbarMessage),

    Toasts(ToastMessage),
    Keybinds(KeybindMessage),
    Panes(PanesMessage),

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
            keybinds: Keybinds::default(),
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
        let subscriptions = vec![self.toasts.subscription(), self.keybinds.subscription()];

        Subscription::batch(subscriptions)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        #[cfg(debug_assertions)]
        println!("{message:?}");

        self.toasts.listen_to_events(&message);

        match message {
            Message::Panes(message) => self.panes.update(message),
            Message::Toasts(message) => return self.toasts.update(message),
            Message::Keybinds(message) => self.keybinds.update(message),

            Message::Inspector(message) => return self.inspector.update(message),
            Message::Layers(message) => return self.layers.update(message),
            Message::Toolbar(message) => return self.toolbar.update(message),

            Message::Scene(message) => {
                let edit = self.scene.update(message);
                self.history.apply(edit);
            }
            Message::History(command) => return self.history.update(command, &mut self.scene),

            Message::Export(process) => match process {
                IoProcess::Start => return save_bytes_async(export_png(&self.scene), "hexmap.png"),
                IoProcess::Cancelled => eprintln!("Export cancelled"),
                IoProcess::Finished(Ok(_)) => eprintln!("Export succeeded"),
                IoProcess::Finished(Err(err)) => eprintln!("Export failed: {err}"),
            },

            Message::Save(process) => match process {
                IoProcess::Start => return save_project_async(&self.scene),
                IoProcess::Cancelled => eprintln!("Project save cancelled"),
                IoProcess::Finished(Ok(_)) => eprintln!("Project save succeeded"),
                IoProcess::Finished(Err(err)) => eprintln!("Project save failed: {err}"),
            },

            Message::Load(process) => match process {
                IoProcess::Start => return load_project_async(),
                IoProcess::Cancelled => eprintln!("Project load cancelled"),
                IoProcess::Finished(Ok(document)) => {
                    self.scene = Scene::from(document);
                    eprintln!("Project load succeeded")
                }
                IoProcess::Finished(Err(err)) => eprintln!("Project load failed: {err}"),
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
