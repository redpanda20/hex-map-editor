use crate::{
    app::Message,
    domain::{
        Layer, Scene,
        edit::{Rename, SetVisible},
        id::LayerId,
        inspect::{ActionHint, Property},
        layer::Inspectable,
    },
    infrastructure::IoProcess,
    ui::widgets::{
        bounded_float_field, bounded_integer_field, colour_field, float_field, inline_text_field,
        integer_field,
    },
};
use iced::{
    Alignment, Element, Length, Task,
    widget::{Row, Text, button, checkbox, column, container, row, rule, space, text},
};
use iced_fonts::lucide;

#[derive(Debug, Clone)]
pub enum InspectorMessage {}

#[derive(Debug, Default, Clone)]
pub struct Inspector {}

impl Inspector {
    pub fn update(&mut self, _message: InspectorMessage) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(
        &'a self,
        scene: &'a Scene,
        active_layer: Option<LayerId>,
    ) -> Element<'a, Message> {
        let Some(layer) = active_layer.and_then(|id| scene.get_layer(id)) else {
            return column![rule::horizontal(1), text("No layer selected"),]
                .height(Length::Fill)
                .width(Length::Fill)
                .spacing(8.0)
                .into();
        };

        let Layer {
            id,
            name,
            visible,
            kind,
        } = layer;

        column![
            container(inline_text_field(name, move |new_name| {
                Rename {
                    id: *id,
                    name: new_name.to_string(),
                }
                .into()
            }))
            .center_x(Length::Fill),
            visible_toggle(*id, visible),
            column(
                kind.properties()
                    .into_iter()
                    .map(|property| Self::view_property(property, *id)),
            )
            .spacing(12),
        ]
        .into()
    }

    fn view_property<'a>(property: Property<'a>, id: LayerId) -> Element<'a, Message> {
        match property {
            Property::Group { info, children } => {
                let row = row(children
                    .into_iter()
                    .map(|property| Self::view_property(property, id)))
                .spacing(12);
                match info {
                    Some(info) => column![text(info.label).style(text::secondary), row]
                        .spacing(4)
                        .into(),
                    None => row.into(),
                }
            }

            Property::ReadOnly {
                info,
                display_value,
            } => column![
                info.map(|info| text(info.label).style(text::secondary)),
                display_value.map(text)
            ]
            .spacing(4)
            .into(),
            Property::Action {
                info,
                display_value: value,
                action_hint,
                action,
            } => column![
                row![
                    text(info.label).style(text::secondary),
                    space::horizontal(),
                    button(render_action_hint(action_hint))
                        .on_press_with(move || Message::Scene((action)(id)))
                        .style(|theme, status| {
                            let mut style = button::subtle(theme, status);

                            if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                                style.text_color = theme.palette().primary
                            }

                            style
                        })
                ]
                .align_y(Alignment::Center),
                // No space is allocated for a None value
                value.map(|value| row![space::horizontal(), text(value)])
            ]
            .spacing(4)
            .into(),

            Property::File {
                info,
                display_value,
                kind,
            } => column![
                text(info.label).style(text::secondary),
                space::horizontal(),
                row![
                    display_value.map(text),
                    space::horizontal(),
                    button("Load").on_press(Message::LoadAsset {
                        caller: id,
                        kind,
                        process: IoProcess::Start
                    })
                ]
                .align_y(Alignment::Center)
            ]
            .into(),

            Property::Boolean {
                info,
                value,
                on_submit,
            } => row![
                text(info.label).style(text::secondary),
                space::horizontal(),
                checkbox(value).on_toggle(move |new| Message::Scene((on_submit)(new, id)))
            ]
            .into(),
            Property::Float {
                info,
                value,
                on_submit,
            } => float_field(info.label, value, move |new| {
                Message::Scene((on_submit)(new, id))
            })
            .into(),
            Property::Integer {
                info,
                value,
                on_submit,
            } => integer_field(info.label, value, move |new| {
                Message::Scene((on_submit)(new, id))
            })
            .into(),
            Property::Colour {
                info,
                value,
                on_submit,
            } => colour_field(info.label, value, move |new| {
                Message::Scene((on_submit)(new, id))
            }),

            Property::BoundedFloat {
                info,
                value,
                range,
                on_submit,
            } => bounded_float_field(info.label, value, range, move |new| {
                Message::Scene((on_submit)(new, id))
            }),
            Property::BoundedInteger {
                info,
                value,
                range,
                on_submit,
            } => bounded_integer_field(info.label, value, range, move |new| {
                Message::Scene((on_submit)(new, id))
            })
            .into(),
        }
    }
}

fn render_action_hint<'a>(hint: ActionHint) -> Text<'a> {
    match hint {
        ActionHint::Text(content) => text(content),
        ActionHint::Refresh => lucide::refresh_cw(),
    }
}

fn visible_toggle<'a>(id: LayerId, visible: &bool) -> Row<'a, Message> {
    let inner = match visible {
        true => row![
            lucide::eye().style(text::secondary),
            text("Visible").style(text::secondary)
        ],
        false => row![
            lucide::eye_off().style(text::secondary),
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
