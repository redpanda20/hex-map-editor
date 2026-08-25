use iced::{
    Element, Length, Task, alignment,
    widget::{Row, button, column, container, row, rule, space, text, text_input},
};
use iced_fonts::bootstrap;

use crate::{
    app::Message,
    domain::{
        Layer, Scene,
        edit::{Rename, SetVisible},
        id::LayerId,
    },
};

#[derive(Debug, Clone)]
pub enum InspectorMessage {
    LayerRename(Option<String>),
    LayerRenameCommit { id: LayerId },
    LayerRenameStart(String),
}

#[derive(Debug, Default, Clone)]
pub struct Inspector {
    active_layer_name: Option<String>,
}

impl Inspector {
    pub fn update(&mut self, message: InspectorMessage) -> Task<Message> {
        match message {
            InspectorMessage::LayerRename(new_name) => self.active_layer_name = new_name,
            InspectorMessage::LayerRenameStart(name) => self.active_layer_name = Some(name),
            InspectorMessage::LayerRenameCommit { id } => {
                if let Some(name) = self.active_layer_name.take() {
                    return Task::done(Message::Scene(Box::new(Rename { id, name })));
                }
            }
        }

        Task::none()
    }

    pub fn view<'a>(
        &self,
        scene: &'a Scene,
        active_layer: Option<LayerId>,
    ) -> Element<'a, Message> {
        let Some(layer) = active_layer.and_then(|id| scene.get_layer(id)) else {
            return container(
                column![rule::horizontal(1), text("No layer selected"),]
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .spacing(8.0)
                    .padding(8.0),
            )
            .style(container::bordered_box)
            .into();
        };

        let Layer {
            id,
            name,
            visible,
            kind: _,
        } = layer;

        container(column![
            name_input(*id, name, &self.active_layer_name).map(Message::Inspector),
            visible_toggle(*id, visible),
            text("Feature currently disabled. Will return soon!")
        ])
        .style(container::bordered_box)
        .into()
    }
}

fn name_input<'a>(
    id: LayerId,
    starting_name: &str,
    name: &Option<String>,
) -> Element<'a, InspectorMessage> {
    if let Some(name) = name {
        row![
            bootstrap::input_cursor().style(text::secondary),
            text_input("Layer name...", name)
                .on_input(|s| InspectorMessage::LayerRename(Some(s)))
                .on_submit(InspectorMessage::LayerRenameCommit { id })
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center)
        ]
        .spacing(4.0)
        .align_y(alignment::Vertical::Center)
    } else {
        row![
            space::horizontal(),
            button(
                row![
                    bootstrap::input_cursor().style(text::secondary),
                    text(starting_name.to_owned()).height(20.0)
                ]
                .spacing(4.0)
                .align_y(alignment::Vertical::Center),
            )
            .on_press(InspectorMessage::LayerRenameStart(starting_name.to_owned()))
            .style(button::text),
            space::horizontal()
        ]
    }
    .into()
}

fn visible_toggle<'a>(id: LayerId, visible: &bool) -> Row<'a, Message> {
    let inner = match visible {
        true => row![
            bootstrap::eye().style(text::secondary),
            text("Visible").style(text::secondary)
        ],
        false => row![
            bootstrap::eye_slash().style(text::secondary),
            text("Hidden").style(text::secondary)
        ],
    }
    .spacing(4.0);
    let toggle = button(inner)
        .style(button::text)
        .on_press(Message::Scene(Box::new(SetVisible {
            id,
            visible: !*visible,
        })));
    row![space::horizontal(), toggle, space::horizontal()]
}
