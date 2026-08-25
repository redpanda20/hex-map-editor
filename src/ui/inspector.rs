use iced::{
    Alignment, Color, Element, Length, Task, alignment,
    widget::{Row, button, column, container, row, rule, slider, space, text, text_input},
};
use iced_fonts::bootstrap;
use rand::random;

use crate::{
    app::Message,
    domain::{
        Layer, LayerInner, Scene,
        edit::{Rename, SetColour, SetNoiseParams, SetNoiseSeed, SetVisible},
        id::LayerId,
        layer::{
            noise::{NoiseParams, PerlinNoiseLayer},
            tiles::SparseTiles,
        },
    },
    ui::colour_picker,
};

#[derive(Debug, Clone)]
pub enum InspectorMessage {
    LayerRename(Option<String>),
    LayerRenameCommit { id: LayerId },
    LayerRenameStart(String),

    ColourChange { colour: Color },
    ColourCommit { id: LayerId },

    NoiseParamsChange { params: NoiseParams },
    NoiseParamCommit { id: LayerId },
}

#[derive(Debug, Default, Clone)]
pub struct Inspector {
    active_layer_name: Option<String>,
    active_colour: Option<Color>,
    active_noise_params: Option<NoiseParams>,
}

impl Inspector {
    pub fn update(&mut self, message: InspectorMessage) -> Task<Message> {
        match message {
            InspectorMessage::LayerRename(new_name) => self.active_layer_name = new_name,
            InspectorMessage::LayerRenameStart(name) => self.active_layer_name = Some(name),
            InspectorMessage::LayerRenameCommit { id } => {
                if let Some(name) = self.active_layer_name.take() {
                    let edit = Box::new(Rename { id, name });
                    return Task::done(Message::Scene(edit));
                }
            }
            InspectorMessage::ColourChange { colour } => self.active_colour = Some(colour),
            InspectorMessage::ColourCommit { id } => {
                if let Some(colour) = self.active_colour.take() {
                    let edit = Box::new(SetColour { layer: id, colour });
                    return Task::done(Message::Scene(edit));
                }
            }
            InspectorMessage::NoiseParamsChange { params } => {
                self.active_noise_params = Some(params)
            }
            InspectorMessage::NoiseParamCommit { id } => {
                if let Some(params) = self.active_noise_params.take() {
                    let edit = Box::new(SetNoiseParams { layer: id, params });
                    return Task::done(Message::Scene(edit));
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
            kind,
        } = layer;

        container(column![
            name_input(*id, name, &self.active_layer_name).map(Message::Inspector),
            visible_toggle(*id, visible),
            match kind {
                LayerInner::Tiles(tiles) => self.details_tiles(*id, tiles),
                LayerInner::Perlin(noise) => self.details_noise(*id, noise),
            },
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

impl Inspector {
    fn details_tiles(&self, id: LayerId, tiles: &SparseTiles) -> Element<'_, Message> {
        let colour = self.active_colour.unwrap_or(tiles.colour);

        column![colour_picker(
            colour,
            |colour| Message::Inspector(InspectorMessage::ColourChange { colour }),
            move |_| Message::Inspector(InspectorMessage::ColourCommit { id }),
        )]
        .padding(8)
        .into()
    }

    fn details_noise(&self, id: LayerId, noise: &PerlinNoiseLayer) -> Element<'_, Message> {
        let NoiseParams {
            threshold,
            frequency,
            octaves,
            persistence,
        } = self.active_noise_params.unwrap_or(noise.get_params());

        let seed = row![
            button(bootstrap::arrow_clockwise())
                .on_press_with(move || {
                    Message::Scene(Box::new(SetNoiseSeed {
                        layer: id,
                        seed: random(),
                    }))
                })
                .style(button::text),
            text(noise.get_seed()).style(text::secondary)
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let scale_control = row![
            text!("{frequency:.2}").style(text::secondary),
            slider(1.0..=20.0, frequency, move |frequency| Message::Inspector(
                InspectorMessage::NoiseParamsChange {
                    params: NoiseParams {
                        threshold,
                        frequency,
                        octaves,
                        persistence
                    }
                }
            ))
            .on_release(Message::Inspector(InspectorMessage::NoiseParamCommit {
                id
            }))
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let threshold_control = row![
            text!("{threshold:.2} / 1.00").style(text::secondary),
            slider(0.0..=1.0, threshold, move |threshold| Message::Inspector(
                InspectorMessage::NoiseParamsChange {
                    params: NoiseParams {
                        threshold,
                        frequency,
                        octaves,
                        persistence
                    }
                }
            ))
            .step(0.01)
            .on_release(Message::Inspector(InspectorMessage::NoiseParamCommit {
                id
            }))
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let octave_control = row![
            text!("{octaves}").style(text::secondary),
            slider(1..=8, octaves as i32, move |octaves| Message::Inspector(
                InspectorMessage::NoiseParamsChange {
                    params: NoiseParams {
                        threshold,
                        frequency,
                        octaves: octaves as usize,
                        persistence
                    }
                }
            ))
            .on_release(Message::Inspector(InspectorMessage::NoiseParamCommit {
                id
            }))
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let persistence_control = row![
            text!("{persistence:.2} / 1.00").style(text::secondary),
            slider(0.0..=1.0, persistence, move |persistence| {
                Message::Inspector(InspectorMessage::NoiseParamsChange {
                    params: NoiseParams {
                        threshold,
                        frequency,
                        octaves,
                        persistence,
                    },
                })
            })
            .step(0.01)
            .on_release(Message::Inspector(InspectorMessage::NoiseParamCommit {
                id
            }))
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        column![
            text("Seed:"),
            seed,
            text("Scale:"),
            scale_control,
            text("Threshold:"),
            threshold_control,
            text("Octaves:"),
            octave_control,
            text("Persistence:"),
            persistence_control
        ]
        .spacing(4)
        .padding(8)
        .into()
    }
}
