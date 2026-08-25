use iced::{
    Element, Subscription, Task, Theme,
    widget::{container, stack},
};

use crate::{
    domain::{
        History, Scene, Tool,
        assets::ImageAsset,
        edit::{EditCommand, SetImage},
        id::LayerId,
    },
    infrastructure::{
        IoProcess, SceneV1, export_png, load_image_async, load_project_async, save_bytes_async,
        save_project_async,
    },
    ui::{
        CanvasEvent, Inspector, InspectorMessage, KeybindMessage, Keybinds, Layers, LayersMessage,
        Panes, PanesMessage, ToastMessage, Toasts, Toolbar, ToolbarMessage, canvas_panel,
    },
};

#[derive(Default)]
pub struct App {
    pub scene: Scene,
    pub history: History,

    pub tool: Tool,
    pub current_layer: Option<LayerId>,

    pub toolbar: Toolbar,
    pub layers: Layers,
    pub inspector: Inspector,

    pub toasts: Toasts,
    pub keybinds: Keybinds,
    pub panes: Panes,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Action {
    SetTool(Tool),
    SetLayer(Option<LayerId>),
    Undo,
    Redo,
    Save,
    Load,
    ExportPng,
}

#[derive(Debug, Clone)]
pub enum Message {
    Action(Action),
    Scene(Box<dyn EditCommand>),

    Canvas(CanvasEvent),
    Inspector(InspectorMessage),
    Layers(LayersMessage),
    Toolbar(ToolbarMessage),

    Toasts(ToastMessage),
    Keybinds(KeybindMessage),
    Panes(PanesMessage),

    LoadAsset {
        caller: LayerId,
        process: IoProcess<ImageAsset>,
    },
    Load(IoProcess<SceneV1>),
    Save(IoProcess<()>),
    Export(IoProcess<()>),
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        (App::default(), Task::none())
    }

    pub fn title(&self) -> String {
        "HexMap Editor".to_string()
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

            Message::Scene(command) => self.history.apply(&mut self.scene, command),

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

            Message::LoadAsset { caller, process } => match process {
                IoProcess::Start => return load_image_async(caller),
                IoProcess::Cancelled => eprintln!("Asset load cancelled"),
                IoProcess::Finished(Ok(asset)) => {
                    let id = self.scene.assets.register_image(asset);
                    let edit = SetImage {
                        layer: caller,
                        image: id,
                    };
                    return Task::done(Message::Scene(Box::new(edit)));
                }
                IoProcess::Finished(Err(err)) => eprintln!("Asset load failed: {err}"),
            },

            Message::Action(action) => match action {
                Action::SetTool(tool) => self.tool = tool,
                Action::SetLayer(layer) => self.current_layer = layer,
                Action::Undo => {
                    self.history.undo(&mut self.scene);
                }
                Action::Redo => {
                    self.history.redo(&mut self.scene);
                }
                Action::Save => return Task::done(Message::Save(IoProcess::Start)),
                Action::Load => return Task::done(Message::Load(IoProcess::Start)),
                Action::ExportPng => return Task::done(Message::Export(IoProcess::Start)),
            },
            Message::Canvas(event) => return event.into_task(&self.current_layer, &self.tool),
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let inspector = |scene| self.inspector.view(scene, self.current_layer);
        let layers = |scene| self.layers.view(scene, self.current_layer);
        let toolbar = |_| self.toolbar.view(self.tool, &self.history);
        let canvas = |scene| canvas_panel(scene, self.tool);

        let grid = self
            .panes
            .view(&self.scene, canvas, inspector, layers, toolbar);

        let toasts = self.toasts.view().map(Message::Toasts);

        container(stack!(grid, toasts))
            .padding(2)
            .style(|theme| container::background(theme.extended_palette().background.base.color))
            .into()
    }
}
