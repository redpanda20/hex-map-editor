use crate::{
    app::Message,
    domain::{
        Inspectable, Layer, LayerInner, Scene,
        edit::{
            Rename, SetImageLockAspectRatio, SetImageOpacity, SetImagePosition, SetImageSize,
            SetVisible,
        },
        id::LayerId,
        inspect::{ActionHint, Property},
        layer::image::ImageLayer,
    },
    infrastructure::IoProcess,
    ui::widgets::{
        bounded_float_field, bounded_integer_field, colour_field, float_field, inline_text_field,
        integer_field,
    },
};
use iced::{
    Alignment, Element, Length, Point, Size, Task,
    widget::{Column, Row, Text, button, checkbox, column, container, row, rule, space, text},
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
            match kind {
                LayerInner::Image(image) => self.details_image(*id, image),
                LayerInner::Perlin(inner) => column(
                    inner
                        .properties()
                        .into_iter()
                        .map(|property| Self::view_property(property, *id)),
                )
                .spacing(12),
                LayerInner::Tiles(inner) => column(
                    inner
                        .properties()
                        .into_iter()
                        .map(|property| Self::view_property(property, *id)),
                )
                .spacing(12),
                LayerInner::Unknown(unknown) => column![text(format!(
                    "Unsupported layer (kind: \"{}\"). It will be kept as-is when you save.",
                    unknown.kind
                ))],
            },
        ]
        .into()
    }

    fn view_property<'a>(property: Property<'a>, id: LayerId) -> Element<'a, Message> {
        match property {
            Property::Action {
                info,
                value,
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

            Property::Text {
                info,
                value,
                on_submit,
            } => todo!(),
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

impl Inspector {
    fn details_image(&self, id: LayerId, layer: &ImageLayer) -> Column<'_, Message> {
        let image = layer.image;
        let Size { width, height } = layer.get_size();
        let Point { x, y } = layer.position;
        let lock_aspect_ratio = layer.lock_aspect_ratio;

        let image_control = row![
            text(
                image
                    .map(|id| format!("{id:?}"))
                    .unwrap_or("No image loaded".into())
            )
            .style(text::secondary),
            space::horizontal(),
            button("Load").on_press(Message::LoadAsset {
                caller: id,
                process: IoProcess::Start
            })
        ]
        .align_y(Alignment::Center);

        let opacity_control = bounded_float_field(
            "Opacity",
            layer.get_opacity() as f64,
            0.0..=1.0,
            move |value| {
                SetImageOpacity {
                    id,
                    opacity: value as f32,
                }
                .into()
            },
        );

        let x_control = float_field("X", x as f64, move |x_value| {
            let position = Point::new(x_value as f32, y);
            SetImagePosition { id, position }.into()
        })
        .vertical();

        let y_control = float_field("Y", y as f64, move |y_value| {
            let position = Point::new(x, y_value as f32);
            SetImagePosition { id, position }.into()
        })
        .vertical();

        let width_control = float_field("Width", width as f64, move |width_value| {
            let size = Size::new(width_value as f32, height);
            SetImageSize { id, size }.into()
        })
        .vertical();

        let height_control = float_field("Height", height as f64, move |height_value| {
            let size = Size::new(width, height_value as f32);
            SetImageSize { id, size }.into()
        })
        .vertical();

        let aspect_ratio_control = row![
            text("Lock aspect ratio").style(text::secondary),
            space::horizontal(),
            checkbox(lock_aspect_ratio).on_toggle(move |value| {
                SetImageLockAspectRatio {
                    id,
                    lock_aspect_ratio: value,
                }
                .into()
            })
        ];

        column![
            image_control,
            opacity_control,
            row![x_control, y_control].spacing(12),
            row![width_control, height_control].spacing(12),
            aspect_ratio_control
        ]
        .spacing(12)
        .padding(8)
        .into()
    }
}
