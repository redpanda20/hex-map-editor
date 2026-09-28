use iced::{
    Alignment, Element, Length, Subscription, Task,
    widget::{button, column, container, row, space, text, tooltip},
};
use iced_fonts::lucide;

use crate::{app::Message, infrastructure::IoProcess};
use crate::{
    infrastructure::{
        Duration, Instant,
        IoProcess::{Cancelled, Finished},
    },
    theme,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
struct Toast {
    pub title: String,
    pub body: String,
    pub lifetime: Instant,
    pub kind: ToastKind,
}

#[derive(Debug, Clone, Copy)]
pub enum ToastKind {
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub enum ToastMessage {
    RemoveToast(usize),
    Tick(Instant),
}

pub struct Toasts {
    toasts: Vec<Toast>,
    timeout: Duration,
}

impl Toasts {
    pub fn listen_to_events(&mut self, message: &Message) {
        match message {
            Message::Export(format, process) => match process {
                IoProcess::Start => self.add_toast(
                    "Exporting",
                    format!("Exporting map to {}...", format.name()),
                    ToastKind::Success,
                ),
                IoProcess::Cancelled => self.add_toast(
                    "Export cancelled",
                    "Export cancelled by user.",
                    ToastKind::Warning,
                ),
                IoProcess::Finished(Ok(_)) => self.add_toast(
                    "Export complete",
                    "Map exported successfully.",
                    ToastKind::Success,
                ),
                IoProcess::Finished(Err(err)) => {
                    self.add_toast("Export failed", err, ToastKind::Error)
                }
            },

            Message::Save(process) => match process {
                IoProcess::Start => {
                    self.add_toast("Saving", "Saving project...", ToastKind::Success)
                }
                IoProcess::Cancelled => self.add_toast(
                    "Save cancelled",
                    "Save cancelled by user.",
                    ToastKind::Warning,
                ),
                IoProcess::Finished(Ok(_)) => self.add_toast(
                    "Save complete",
                    "Project saved successfully.",
                    ToastKind::Success,
                ),
                IoProcess::Finished(Err(err)) => {
                    self.add_toast("Save Failed", err, ToastKind::Error)
                }
            },

            Message::Load(process) => match process {
                IoProcess::Start => {
                    self.add_toast("Opening", "Opening project...", ToastKind::Success)
                }
                Cancelled => self.add_toast(
                    "Open cancelled",
                    "Open cancelled by user.",
                    ToastKind::Warning,
                ),
                Finished(Err(err)) => self.add_toast("Project load failed", err, ToastKind::Error),
                // Finished(Ok) ommited. Loading a project visually changes the active scene
                Finished(Ok(_)) => {}
            },

            Message::LoadAsset { process, .. } => match process {
                IoProcess::Start => {
                    self.add_toast("Opening asset", "Opening user asset...", ToastKind::Success)
                }
                Cancelled => self.add_toast(
                    "Asset upload cancelled",
                    "User cancelled loading asset.",
                    ToastKind::Warning,
                ),
                Finished(Err(err)) => self.add_toast("Failed to load asset", err, ToastKind::Error),
                // Case omitted. Loaded asset immediately binds to image layer
                Finished(Ok(_)) => {}
            },

            _ => (),
        }
    }

    pub fn add_toast(
        &mut self,
        title: impl Into<String>,
        body: impl Into<String>,
        kind: ToastKind,
    ) {
        let toast = Toast {
            title: title.into(),
            body: body.into(),
            lifetime: Instant::now() + self.timeout,
            kind,
        };
        self.toasts.push(toast);
    }
}

impl Toasts {
    pub fn subscription(&self) -> Subscription<Message> {
        if let Some(earliest) = self.toasts.iter().min_by_key(|toast| toast.lifetime) {
            iced::time::every(earliest.lifetime - Instant::now())
                .map(ToastMessage::Tick)
                .map(Message::Toasts)
        } else {
            Subscription::none()
        }
    }

    pub fn update(&mut self, toast_event: ToastMessage) -> Task<Message> {
        match toast_event {
            ToastMessage::RemoveToast(index) => {
                self.toasts.remove(index);
            }
            ToastMessage::Tick(instant) => {
                self.toasts.retain(|toast| toast.lifetime > instant);
            }
        };

        Task::none()
    }

    pub fn view(&self) -> Element<'_, ToastMessage> {
        let toasts: Vec<Element<'_, ToastMessage>> = self
            .toasts
            .iter()
            .enumerate()
            .map(|(i, t)| toast_bar(i, t))
            .collect();

        container(column(toasts).spacing(4.0))
            .align_left(Length::Fill)
            .align_bottom(Length::Fill)
            .padding([32.0, 64.0])
            .into()
    }
}
impl Default for Toasts {
    fn default() -> Self {
        Self {
            toasts: Vec::new(),
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

fn toast_bar<'a>(index: usize, toast: &'a Toast) -> Element<'a, ToastMessage> {
    let Toast {
        title, body, kind, ..
    } = toast;
    let status_colour = match kind {
        ToastKind::Success => theme::SUCCESS,
        ToastKind::Warning => theme::WARNING,
        ToastKind::Error => theme::DANGER,
    };

    let content = container(
        row![
            theme::toast_marker(status_colour),
            text(title),
            space::horizontal(),
            button(lucide::x()).on_press(ToastMessage::RemoveToast(index))
        ]
        .spacing(8)
        .height(Length::Shrink)
        .align_y(Alignment::Center),
    )
    .style(theme::toast)
    .padding(4)
    .max_width(240.0);

    let body = container(text(body)).padding(4).style(theme::modal);

    tooltip(content, body, tooltip::Position::Right).into()
}
