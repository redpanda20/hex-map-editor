use iced::{
    Element, Length, Subscription, Task,
    widget::{button, column, container, row, space, text, tooltip},
};
use iced_fonts::bootstrap;

use crate::infrastructure::{
    Duration, Instant,
    IoProcess::{Cancelled, Finished},
};
use crate::{app::Message, infrastructure::IoProcess};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
struct Toast {
    pub title: String,
    pub body: String,
    pub lifetime: Instant,
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
            Message::Export(process) => match process {
                IoProcess::Start => self.add_toast("Exporting", "Exporting map to PNG..."),
                IoProcess::Cancelled => {
                    self.add_toast("Export cancelled", "Export cancelled by user.")
                }
                IoProcess::Finished(Ok(_)) => {
                    self.add_toast("Export complete", "Map exported successfully.")
                }
                IoProcess::Finished(Err(err)) => self.add_toast("Export failed", err),
            },

            Message::Save(process) => match process {
                IoProcess::Start => self.add_toast("Saving", "Saving project..."),
                IoProcess::Cancelled => self.add_toast("Save cancelled", "Save cancelled by user."),
                IoProcess::Finished(Ok(_)) => {
                    self.add_toast("Save complete", "Project saved successfully.")
                }
                IoProcess::Finished(Err(err)) => self.add_toast("Save Failed", err),
            },

            Message::Load(process) => match process {
                IoProcess::Start => self.add_toast("Opening", "Opening project..."),
                Cancelled => self.add_toast("Open cancelled", "Open cancelled by user."),
                Finished(Err(err)) => self.add_toast("Project load failed", err),
                // Finished(Ok) ommited. Loading a project visually changes the active scene
                Finished(Ok(_)) => {}
            },

            Message::LoadAsset { caller: _, process } => match process {
                IoProcess::Start => self.add_toast("Opening asset", "Opening user asset..."),
                Cancelled => {
                    self.add_toast("Asset upload cancelled", "User cancelled loading asset.")
                }
                Finished(Err(err)) => self.add_toast("Failed to load asset", err),
                // Case omitted. Loaded asset immediately binds to image layer
                Finished(Ok(_)) => {}
            },

            _ => (),
        }
    }

    pub fn add_toast(&mut self, title: impl Into<String>, body: impl Into<String>) {
        self.toasts.push(Toast {
            title: title.into(),
            body: body.into(),
            lifetime: Instant::now() + self.timeout,
        });
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
            .map(|(index, toast)| {
                tooltip(
                    container(row![
                        text(toast.title.as_str()),
                        space::horizontal(),
                        button(bootstrap::x_lg()).on_press(ToastMessage::RemoveToast(index))
                    ])
                    .padding(4.0)
                    .style(container::rounded_box)
                    .max_width(200.0),
                    container(text(toast.body.as_str()))
                        .padding(4.0)
                        .style(container::rounded_box),
                    tooltip::Position::Right,
                )
                .into()
            })
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
