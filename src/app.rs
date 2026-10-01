use iced::{
    Element, Size, Subscription, Task, Theme,
    widget::{column, container, row, stack},
};

use crate::{
    domain::{
        History, PrintSettings, Scene, Tool,
        assets::{FileAsset, FileKind},
        edit::{EditCommand, SetImageAndSize},
        id::LayerId,
    },
    infrastructure::{
        Document, ExportFormat, ExportSettings, IoProcess, export_pdf, export_png,
        load_asset_async, load_project_async, save_bytes_async, save_project_async,
    },
    theme,
    ui::{
        About, AboutMessage, CanvasEvent, ExportDialog, ExportDialogMessage, Inspector,
        InspectorMessage, KeybindMessage, Keybinds, Layers, LayersMessage, Menubar, Panes,
        PanesMessage, ToastMessage, Toasts, Toolbar, ToolbarMessage, canvas_panel,
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
    pub about: About,
    pub export: ExportDialog,
    pub keybinds: Keybinds,
    pub panels: Panes,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Action {
    SetTool(Tool),
    SetLayer(Option<LayerId>),
    Undo,
    Redo,
    Save,
    Load,
    Export,
    About,
    ExportAs(ExportFormat),
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
    ExportDialog(ExportDialogMessage),
    About(AboutMessage),
    Keybinds(KeybindMessage),
    Panes(PanesMessage),

    LoadAsset {
        caller: LayerId,
        kind: FileKind,
        process: IoProcess<FileAsset>,
    },
    Load(IoProcess<Document>),
    Save(IoProcess<()>),
    Export(ExportSettings, IoProcess<()>),
}

impl<T> From<T> for Message
where
    T: EditCommand + 'static,
{
    fn from(value: T) -> Self {
        Self::Scene(Box::new(value))
    }
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        (App::default(), Task::none())
    }

    pub fn title(&self) -> String {
        "HexMap Editor".to_string()
    }

    pub fn theme(&self) -> Option<Theme> {
        let theme = theme::theme();
        Some(theme)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let subscriptions = vec![self.toasts.subscription(), self.keybinds.subscription()];

        Subscription::batch(subscriptions)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.toasts.listen_to_events(&message);

        match message {
            Message::Panes(message) => self.panels.update(message),
            Message::Toasts(message) => return self.toasts.update(message),
            Message::ExportDialog(message) => return self.export.update(message),
            Message::About(message) => return self.about.update(message),
            Message::Keybinds(message) => self.keybinds.update(message),

            Message::Inspector(message) => return self.inspector.update(message),
            Message::Layers(message) => return self.layers.update(message),
            Message::Toolbar(message) => return self.toolbar.update(message),

            Message::Scene(command) => self.history.apply(&mut self.scene, command),

            Message::Export(settings, process) => match process {
                IoProcess::Start => {
                    let bytes = match settings {
                        ExportSettings::Png(hex_size) => export_png(&self.scene, hex_size),
                        ExportSettings::Pdf(print_settings) => {
                            export_pdf(&self.scene, print_settings)
                        }
                    };
                    return save_bytes_async(
                        bytes,
                        &format!("hexmap.{}", settings.get_format().extension()),
                        settings,
                    );
                }
                IoProcess::Cancelled => eprintln!("Export cancelled"),
                IoProcess::Finished(Ok(_)) => eprintln!("Export succeeded"),
                IoProcess::Finished(Err(err)) => eprintln!("Export failed: {err}"),
            },

            Message::Save(process) => match process {
                IoProcess::Start => return save_project_async(&self.scene),
                IoProcess::Cancelled => eprintln!("Scene save cancelled"),
                IoProcess::Finished(Ok(_)) => eprintln!("Scene save succeeded"),
                IoProcess::Finished(Err(err)) => eprintln!("Scene save failed: {err}"),
            },

            Message::Load(process) => match process {
                IoProcess::Start => return load_project_async(),
                IoProcess::Cancelled => eprintln!("Scene load cancelled"),
                IoProcess::Finished(Ok(document)) => {
                    self.scene = document.into_scene();
                    // Clears everything that could refer to previous scene.
                    self.current_layer = None;
                    self.layers = Layers::default();
                    self.history = History::default();

                    eprintln!("Scene load succeeded")
                }
                IoProcess::Finished(Err(err)) => eprintln!("Scene load failed: {err}"),
            },

            Message::LoadAsset {
                caller,
                kind,
                process,
            } => match process {
                IoProcess::Start => return load_asset_async(caller, kind),
                IoProcess::Cancelled => eprintln!("Asset load cancelled"),
                IoProcess::Finished(Ok(asset)) => {
                    let edit = match asset {
                        FileAsset::Image(image_asset) => {
                            let size = Size {
                                width: image_asset.width as f32,
                                height: image_asset.height as f32,
                            };
                            let id = self.scene.assets.register_image(image_asset);
                            SetImageAndSize {
                                layer: caller,
                                image: Some(id),
                                size,
                            }
                        }
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
                Action::Export => {
                    return Task::done(Message::ExportDialog(ExportDialogMessage::Show));
                }
                Action::About => return Task::done(Message::About(AboutMessage::Show)),
                Action::ExportAs(format) => {
                    let export_settings = match format {
                        ExportFormat::Png => ExportSettings::Png(100.0),
                        ExportFormat::Pdf => ExportSettings::Pdf(PrintSettings::DEFAULT),
                    };
                    return Task::done(Message::Export(export_settings, IoProcess::Start));
                }
            },
            Message::Canvas(event) => return event.into_task(&self.current_layer, &self.tool),
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let toolbar = self.toolbar.view(self.tool, &self.history);
        let canvas = canvas_panel(&self.scene, self.tool);
        let viewport = row![toolbar, canvas];

        let inspector = |scene| self.inspector.view(scene, self.current_layer);
        let layers = |scene| self.layers.view(scene, self.current_layer);
        let panels = self.panels.view(&self.scene, inspector, layers);

        let body = row![viewport, panels];

        let menubar = Menubar.view(&self.keybinds);
        let app = column![menubar, body].spacing(4);

        let toasts = self.toasts.view().map(Message::Toasts);
        let about = self.about.view().map(|el| el.map(Message::About));
        let export_dialog = self.export.view();

        container(stack![app, about, export_dialog, toasts])
            .padding(4)
            .style(|_| container::background(theme::BG))
            .into()
    }
}
