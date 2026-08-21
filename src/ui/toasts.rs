use iced::{
    Element, Length, Subscription, Task,
    widget::{button, column, container, row, space, text, tooltip},
};
use iced_fonts::bootstrap;

use crate::infrastructure::{Duration, Instant};
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
            Message::Export(IoProcess::Start) => self.add_toast(
                "Exporting".to_string(),
                "Exporting map to PNG...".to_string(),
            ),
            Message::Export(IoProcess::Cancelled) => self.add_toast(
                "Export Cancelled".to_string(),
                "Export cancelled by user.".to_string(),
            ),
            Message::Export(IoProcess::Finished(Ok(_))) => self.add_toast(
                "Export Complete".to_string(),
                "Map exported successfully.".to_string(),
            ),
            Message::Export(IoProcess::Finished(Err(err))) => {
                self.add_toast("Export Failed".to_string(), err.clone())
            }

            // TODO: Evaluate if Finished(Err) is needed
            Message::Save(IoProcess::Start) => {
                self.add_toast("Saving".to_string(), "Saving project...".to_string())
            }
            Message::Save(IoProcess::Cancelled) => self.add_toast(
                "Save Cancelled".to_string(),
                "Save cancelled by user.".to_string(),
            ),
            Message::Save(IoProcess::Finished(Ok(_))) => self.add_toast(
                "Save Complete".to_string(),
                "Project saved successfully.".to_string(),
            ),
            Message::Save(IoProcess::Finished(Err(err))) => {
                self.add_toast("Save Failed".to_string(), err.clone())
            }

            Message::Load(IoProcess::Start) => {
                self.add_toast("Opening".to_string(), "Opening project...".to_string())
            }
            Message::Load(IoProcess::Cancelled) => self.add_toast(
                "Open Cancelled".to_string(),
                "Open cancelled by user.".to_string(),
            ),
            Message::Load(IoProcess::Finished(Ok(_))) => self.add_toast(
                "Project Loaded".to_string(),
                "Project loaded successfully.".to_string(),
            ),
            Message::Load(IoProcess::Finished(Err(err))) => {
                self.add_toast("Open Failed".to_string(), err.clone())
            }
            _ => (),
        }
    }

    pub fn add_toast(&mut self, title: String, body: String) {
        self.toasts.push(Toast {
            title,
            body,
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
