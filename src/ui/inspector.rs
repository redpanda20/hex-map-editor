use crate::{
    app::Message,
    domain::{
        Layer, LayerInner, Scene,
        edit::{
            Rename, SetImageOpacity, SetImagePosition, SetImageSize, SetNoiseFrequency,
            SetNoiseOctaves, SetNoisePersistence, SetNoiseSeed, SetNoiseThreshold, SetTilesColour,
            SetVisible,
        },
        id::LayerId,
        inspect::{BoolProperty, BoundedFloatProperty, Property, TextProperty},
        layer::{
            image::ImageLayer,
            noise::{NoiseParams, PerlinNoiseLayer},
            tiles::SparseTiles,
        },
    },
    infrastructure::IoProcess,
    ui::widgets::{bounded_float_field, bounded_integer_field, colour_field, inline_text_field},
};
use iced::{
    Alignment, Element, Length, Padding, Point, Size, Task,
    widget::{Row, button, checkbox, column, container, row, rule, space, text, text_input},
};
use iced_fonts::bootstrap;
use rand::random;

#[derive(Debug, Clone)]
pub enum InspectorMessage {
    Clear,

    ImageSizeChange(Option<Size>),
    ImageSizeCommit { id: LayerId },

    ImagePositionChange(Option<Point>),
    ImagePositionCommit { id: LayerId },
}

#[derive(Debug, Default, Clone)]
pub struct Inspector {
    active_size: Option<Size>,
    active_position: Option<Point>,
}

impl Inspector {
    pub fn update(&mut self, message: InspectorMessage) -> Task<Message> {
        match message {
            InspectorMessage::Clear => {
                self.active_position = None;
                self.active_size = None;
            }

            InspectorMessage::ImagePositionChange(position) => self.active_position = position,
            InspectorMessage::ImagePositionCommit { id } => {
                if let Some(position) = self.active_position {
                    return Task::done(
                        SetImagePosition {
                            layer: id,
                            position,
                        }
                        .into(),
                    )
                    .chain(Task::done(Message::Inspector(InspectorMessage::Clear)));
                }
            }
            InspectorMessage::ImageSizeChange(size) => self.active_size = size,
            InspectorMessage::ImageSizeCommit { id } => {
                if let Some(size) = self.active_size {
                    return Task::done(SetImageSize { layer: id, size }.into())
                        .chain(Task::done(Message::Inspector(InspectorMessage::Clear)));
                }
            }
        }

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
                LayerInner::Tiles(tiles) => self.details_tiles(*id, tiles),
                LayerInner::Perlin(noise) => self.details_noise(*id, noise),
                LayerInner::Image(image) => self.details_image(*id, image),
                LayerInner::Unknown(unknown) => text(format!(
                    "Unsupported layer (kind: \"{}\"). It will be kept as-is when you save.",
                    unknown.kind
                ))
                .into(),
            },
        ]
        .into()
    }

    #[allow(unused)]
    fn view_property<'a>(property: Property<'a>, id: LayerId) -> Element<'a, Message> {
        const FIELD_INDENT: Length = Length::Fixed(80.0);
        match property {
            Property::Text(TextProperty {
                name,
                value,
                on_submit,
            }) => {
                let field = inline_text_field(value, move |name| {
                    Message::Scene(on_submit(name.to_string(), id))
                });
                row![text(name).style(text::secondary).width(FIELD_INDENT), field]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into()
            }
            Property::Bool(BoolProperty {
                name,
                value,
                on_submit,
            }) => {
                let field = checkbox(value)
                    .on_toggle(move |enabled| Message::Scene(on_submit(enabled, id)));
                row![text(name).style(text::secondary).width(FIELD_INDENT), field]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into()
            }
            Property::BoundedFloat(BoundedFloatProperty {
                name,
                value,
                range,
                on_submit,
            }) => bounded_float_field(name, value, range, move |value| {
                Message::Scene(on_submit(value, id))
            }),
        }
    }
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

impl Inspector {
    fn details_tiles(&self, id: LayerId, tiles: &SparseTiles) -> Element<'_, Message> {
        column![colour_field("Colour", tiles.colour, move |colour| {
            SetTilesColour { id, colour }.into()
        })]
        .padding(8)
        .into()
    }

    fn details_noise(&self, id: LayerId, noise: &PerlinNoiseLayer) -> Element<'_, Message> {
        let NoiseParams {
            threshold,
            frequency,
            octaves,
            persistence,
        } = noise.get_params();

        let seed = column![
            row![
                text("Seed").style(text::secondary),
                space::horizontal(),
                button(bootstrap::arrow_clockwise())
                    .on_press_with(move || {
                        Message::Scene(Box::new(SetNoiseSeed {
                            layer: id,
                            seed: random(),
                        }))
                    })
                    .style(button::text)
            ]
            .align_y(Alignment::Center),
            row![space::horizontal(), text(noise.get_seed())]
        ];

        let scale_control =
            bounded_float_field("Scale", frequency as f64, 1.0..=20.0, move |value| {
                Message::Scene(Box::new(SetNoiseFrequency {
                    id,
                    frequency: value as f32,
                }))
            });

        let threshold_control =
            bounded_float_field("Threshold", threshold as f64, 0.0..=1.0, move |value| {
                Message::Scene(Box::new(SetNoiseThreshold {
                    id,
                    threshold: value as f32,
                }))
            });

        let octave_control =
            bounded_integer_field("Octaves", octaves as u64, 1..=8, move |value| {
                Message::Scene(Box::new(SetNoiseOctaves {
                    id,
                    octaves: value as usize,
                }))
            });

        let persistence_control =
            bounded_float_field("Persistence", persistence as f64, 0.0..=1.0, move |value| {
                Message::Scene(Box::new(SetNoisePersistence {
                    id,
                    persistence: value as f32,
                }))
            });

        column![
            seed,
            scale_control,
            threshold_control,
            octave_control,
            persistence_control
        ]
        .spacing(12)
        .padding(8)
        .into()
    }

    fn details_image(&self, id: LayerId, layer: &ImageLayer) -> Element<'_, Message> {
        let Point { x, y } = self.active_position.unwrap_or(layer.position);
        let Size { width, height } = self.active_size.unwrap_or(layer.size);

        let image_control = row![
            text(
                layer
                    .image
                    .map(|id| format!("{id:?}"))
                    .unwrap_or("No image loaded".into())
            )
            .style(text::secondary),
            button("Load").on_press(Message::LoadAsset {
                caller: id,
                process: IoProcess::Start
            })
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding(Padding::default().bottom(8));

        let opacity_control = bounded_float_field(
            "Opacity",
            layer.get_opacity() as f64,
            0.0..=1.0,
            move |value| {
                SetImageOpacity {
                    layer: id,
                    opacity: value as f32,
                }
                .into()
            },
        );

        let x_control = text_input("0.0", &x.to_string())
            .on_input(move |x_maybe| {
                let pos = x_maybe.parse::<f32>().ok().map(|x| Point { x, y });
                Message::Inspector(InspectorMessage::ImagePositionChange(pos))
            })
            .on_submit(Message::Inspector(InspectorMessage::ImagePositionCommit {
                id,
            }))
            .width(Length::Fill);

        let y_control = text_input("0.0", &y.to_string())
            .on_input(move |y_maybe| {
                let pos = y_maybe.parse::<f32>().ok().map(|y| Point { x, y });
                Message::Inspector(InspectorMessage::ImagePositionChange(pos))
            })
            .on_submit(Message::Inspector(InspectorMessage::ImagePositionCommit {
                id,
            }))
            .width(Length::Fill);

        let position_control = row![
            text("X:").style(text::secondary),
            x_control,
            text("Y:").style(text::secondary),
            y_control
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding(Padding::default().bottom(8));

        let width_control = text_input("0.0", &width.to_string())
            .on_input(move |w_maybe| {
                let size = w_maybe
                    .parse::<f32>()
                    .ok()
                    .map(|width| Size { width, height });
                Message::Inspector(InspectorMessage::ImageSizeChange(size))
            })
            .on_submit(Message::Inspector(InspectorMessage::ImageSizeCommit { id }))
            .width(Length::Fill);

        let height_control = text_input("0.0", &height.to_string())
            .on_input(move |h_maybe| {
                let size = h_maybe
                    .parse::<f32>()
                    .ok()
                    .map(|height| Size { width, height });
                Message::Inspector(InspectorMessage::ImageSizeChange(size))
            })
            .on_submit(Message::Inspector(InspectorMessage::ImageSizeCommit { id }))
            .width(Length::Fill);

        let size_control = row![
            text("Width:").style(text::secondary),
            width_control,
            text("Height:").style(text::secondary),
            height_control
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding(Padding::default().bottom(8));

        column![
            image_control,
            opacity_control,
            text("Position:"),
            position_control,
            text("Size:"),
            size_control,
        ]
        .spacing(4)
        .padding(8)
        .into()
    }
}
