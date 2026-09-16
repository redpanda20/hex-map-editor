use crate::{
    app::Message,
    domain::{
        Layer, LayerInner, Scene,
        edit::{
            Rename, SetImageLockAspectRatio, SetImageOpacity, SetImagePosition, SetImageSize,
            SetNoiseFrequency, SetNoiseOctaves, SetNoisePersistence, SetNoiseSeed,
            SetNoiseThreshold, SetTilesColour, SetVisible,
        },
        id::LayerId,
        inspect::{
            BoolProperty, BoundedFloatProperty, BoundedIntegerProperty, FloatProperty, Property,
            TextProperty,
        },
        layer::{
            image::ImageLayer,
            noise::{NoiseParams, PerlinNoiseLayer},
            tiles::SparseTiles,
        },
    },
    infrastructure::IoProcess,
    ui::widgets::{
        bounded_float_field, bounded_integer_field, colour_field, float_field, inline_text_field,
    },
};
use iced::{
    Alignment, Element, Length, Point, Size, Task,
    widget::{Row, button, checkbox, column, container, row, rule, space, text},
};
use iced_fonts::lucide;
use rand::random;

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
        match property {
            Property::Text(TextProperty {
                name,
                value,
                on_submit,
            }) => {
                let field = inline_text_field(value, move |name| {
                    Message::Scene(on_submit(name.to_string(), id))
                });
                row![
                    text(name).style(text::secondary),
                    space::horizontal(),
                    field
                ]
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
                    .on_toggle(move |enabled| Message::Scene(on_submit(enabled, id)))
                    .width(80);
                row![
                    text(name).style(text::secondary),
                    space::horizontal(),
                    field
                ]
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
            Property::Float(FloatProperty {
                name,
                value,
                on_submit,
            }) => float_field(name, value, move |value| {
                Message::Scene(on_submit(value, id))
            })
            .into(),
            Property::BoundedInteger(BoundedIntegerProperty {
                name,
                value,
                range,
                on_submit,
            }) => bounded_integer_field(name, value, range, move |value| {
                Message::Scene(on_submit(value, id))
            })
            .into(),
        }
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

        let seed_control = column![
            row![
                text("Seed").style(text::secondary),
                space::horizontal(),
                button(lucide::refresh_cw())
                    .on_press_with(move || SetNoiseSeed {
                        layer: id,
                        seed: random(),
                    }
                    .into())
                    .style(|theme, status| {
                        let mut style = button::subtle(theme, status);

                        if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                            style.text_color = theme.palette().primary
                        }

                        style
                    })
            ]
            .align_y(Alignment::Center),
            row![space::horizontal(), text(noise.get_seed())]
        ]
        .spacing(4);

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
            seed_control,
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
