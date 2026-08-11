use iced::{
    Element, Subscription, Task, Theme,
    widget::{container, stack},
};

use crate::{
    domain::{LayerMessage, LayerType, Scene, SceneCommand, Tool},
    infrastructure::{
        SceneV1, export_png, load_project_async, save_bytes_async, save_project_async,
    },
    ui::{
        Inspector, InspectorMessage, PaneType, Panes, PanesMessage, ToastMessage, Toasts,
        canvas_panel, default_pane_config, inspector_panel, layer_stack_panel, toast_widget,
        toolbar_panel,
    },
};

#[derive(Default, Debug)]
pub struct EditorState {
    pub active_layer_name: Option<String>,
    pub active_layer_type: LayerType,
}

pub struct App {
    pub scene: Scene,

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

    Scene(SceneCommand),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            scene: Scene::new(),
            canvas: todo!(),
            toolbar: todo!(),
            layers: todo!(),
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

            Message::Scene(command) => self.scene.update(command),
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let canvas = canvas_panel(&self.scene);
        let inspector = self.inspector.view(&self.scene);

        let grid = self.panes.view(canvas, inspector);
        let toasts = self.toasts.view().map(Message::Toasts);
        // let grid = pane_grid(&self.panes, |_id, state, _is_maximised| {

        //     let inner: Element<'_, Message> = match state {
        //         PaneType::Toolbar => toolbar_panel(&self.active_tool),
        //         PaneType::LayerStack => layer_stack_panel(&self.layers, &self.editor_state),
        //         PaneType::Canvas => canvas_panel(&self.layers, &self.active_tool),
        //         PaneType::Inspector => inspector_panel(&self.layers, &self.editor_state),
        //     };

        //     pane_grid::Content::new(inner)
        // })
        // .on_resize(10, Message::PaneResized)
        // .spacing(2);

        container(stack!(grid, toasts))
            .padding(2)
            .style(|theme| container::background(theme.extended_palette().background.base.color))
            .into()
    }
}
