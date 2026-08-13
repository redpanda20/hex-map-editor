use iced::{
    Color, Element, Length, Task, alignment, padding,
    widget::{
        Column, Row, button, column, container, responsive, row, rule, slider, space, text,
        text_input,
    },
};
use iced_fonts::bootstrap;

use crate::{
    app::Message,
    domain::{Layer, NoiseOctaves, PerlinNoiseLayer, Scene, SceneCommand, SparseTiles},
};

#[derive(Debug, Clone)]
pub enum InspectorMessage {
    LayerRename(Option<String>),
    LayerRenameCommit(usize),
    LayerRenameStart(String),
}

pub struct Inspector {
    active_layer_name: Option<String>,
}

impl Inspector {
    pub fn new() -> Self {
        Self {
            active_layer_name: None,
        }
    }

    pub fn update(&mut self, message: InspectorMessage) -> Task<Message> {
        match message {
            InspectorMessage::LayerRename(new_name) => self.active_layer_name = new_name,
            InspectorMessage::LayerRenameStart(name) => self.active_layer_name = Some(name),
            InspectorMessage::LayerRenameCommit(index) => {
                if let Some(name) = &mut self.active_layer_name {
                    return Task::done(Message::Scene(SceneCommand::EditLayerName(
                        index,
                        name.clone(),
                    )));
                }
            }
        }

        Task::none()
    }

    pub fn view<'a>(&self, scene: &'a Scene) -> Element<'a, Message> {
        let Some((layer_id, layer)) = scene
            .active_layer
            .and_then(|id| scene.inner.get(id).map(|layer| (id, layer)))
        else {
            return container(
                column![rule::horizontal(1), text("No active content")]
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .spacing(8.0)
                    .padding(8.0),
            )
            .style(container::bordered_box)
            .into();
        };

        let Layer {
            name,
            visible,
            inner,
        } = layer;

        let starting_name = name;
        let name = &self.active_layer_name;

        let content = match inner {
            crate::domain::LayerInner::Tiles(sparse_tiles) => {
                sparse_tile_details(layer_id, starting_name, name, visible, sparse_tiles)
            }
            crate::domain::LayerInner::InvertedTiles(sparse_tiles) => {
                sparse_tile_details(layer_id, starting_name, name, visible, sparse_tiles)
            }
            crate::domain::LayerInner::Perlin(content) => {
                perlin_noise_details(layer_id, starting_name, name, visible, content)
            }
        };

        container(
            column![rule::horizontal(1), content]
                .height(Length::Fill)
                .width(Length::Fill)
                .spacing(8.0)
                .padding(8.0),
        )
        .style(container::bordered_box)
        .into()
    }
}

fn sparse_tile_details<'a>(
    layer_id: usize,
    starting_name: &String,
    name: &Option<String>,
    visible: &bool,
    tiles: &SparseTiles,
) -> Column<'a, Message> {
    column![
        name_input(layer_id, starting_name, name).map(Message::Inspector),
        visible_toggle(layer_id, visible),
        colour_panel(layer_id, tiles.get_colour())
    ]
}

fn perlin_noise_details<'a>(
    layer_id: usize,
    starting_name: &String,
    name: &Option<String>,
    visible: &bool,
    content: &PerlinNoiseLayer,
) -> Column<'a, Message> {
    let PerlinNoiseLayer {
        seed,
        scale,
        threshold,
        octaves,
        ..
    } = content;
    let seed_controls = row![
        button(bootstrap::arrow_clockwise())
            .style(button::text)
            .on_press_with(move || {
                let new_seed = rand::random();
                Message::Scene(SceneCommand::EditLayerSeed(layer_id, new_seed))
            }),
        text(seed).style(text::secondary)
    ]
    .spacing(4.0)
    .align_y(alignment::Vertical::Center);

    let inverse_scale = 1.0 / scale;
    let scale_controls = row![
        text!("{inverse_scale:.2}").style(text::secondary),
        slider(1.0..=20.0, inverse_scale, move |value| {
            Message::Scene(SceneCommand::EditLayerScale(layer_id, 1.0 / value))
        })
    ]
    .spacing(8.0)
    .align_y(alignment::Vertical::Center);

    let threshold_controls = row![
        text!("{threshold:.2} / 1.00").style(text::secondary),
        slider(0.0..=1.0, *threshold, move |value| {
            Message::Scene(SceneCommand::EditLayerThreshold(layer_id, value))
        })
        .step(0.01)
    ]
    .spacing(8.0)
    .align_y(alignment::Vertical::Center);

    let (octave_count, persistence) = match octaves {
        NoiseOctaves::One => (1, 0.0),
        NoiseOctaves::Many { count, persistence } => (*count as i32, *persistence),
    };
    let mut octave_controls = column![];

    octave_controls = octave_controls.push(text("Count:"));
    octave_controls = octave_controls.push(
        row![
            text!("{octave_count}").style(text::secondary),
            slider(1..=8, octave_count, move |value| {
                Message::Scene(SceneCommand::EditLayerOctaves(layer_id, value as usize))
            })
        ]
        .spacing(8.0)
        .align_y(alignment::Vertical::Center),
    );

    if octave_count > 1 {
        octave_controls = octave_controls.push(text("Persistence:"));
        octave_controls = octave_controls.push(
            row![
                text!("{persistence:.2} / 1.00").style(text::secondary),
                slider(1.0..=1.0, persistence, move |value| {
                    Message::Scene(SceneCommand::EditLayerPersistence(layer_id, value))
                })
                .step(0.1)
            ]
            .spacing(8.0)
            .align_y(alignment::Vertical::Center),
        );
    }

    column![
        name_input(layer_id, starting_name, name).map(Message::Inspector),
        visible_toggle(layer_id, visible),
        text("Noise"),
        rule::horizontal(1),
        text("Seed:"),
        seed_controls,
        text("Scale:"),
        scale_controls,
        text("Threshold:"),
        threshold_controls,
        column![text("Octaves:"), rule::horizontal(1), octave_controls,]
            .padding(padding::top(16.0))
    ]
}

fn name_input<'a>(
    layer_id: usize,
    starting_name: &String,
    name: &Option<String>,
) -> Element<'a, InspectorMessage> {
    if let Some(name) = name {
        row![
            bootstrap::input_cursor().style(text::secondary),
            text_input("Layer name...", &name)
                .on_input(|s| InspectorMessage::LayerRename(Some(s)))
                .on_submit(InspectorMessage::LayerRenameCommit(layer_id))
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
                    text(starting_name.clone()).height(20.0)
                ]
                .spacing(4.0)
                .align_y(alignment::Vertical::Center),
            )
            .on_press(InspectorMessage::LayerRenameStart(starting_name.clone()))
            .style(button::text),
            space::horizontal()
        ]
    }
    .into()
}

fn visible_toggle<'a>(layer_id: usize, visible: &bool) -> Row<'a, Message> {
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
    let toggle = button(inner).style(button::text).on_press(Message::Scene(
        SceneCommand::EditLayerVisibility(layer_id, !visible),
    ));
    row![space::horizontal(), toggle, space::horizontal()]
}

fn colour_panel<'a>(layer_id: usize, active_colour: Color) -> Column<'a, Message> {
    let square = responsive(|size| {
        let new_size = size.ratio(1.0);
        space().width(new_size.width).height(new_size.height).into()
    });

    let colour_preview = container(square)
        .height(Length::Shrink)
        .style(move |_theme| container::background(active_colour));

    let Color { r, g, b, a } = active_colour;

    let red_slider: Element<'_, Message> = slider(0.0..=1.0, r, move |value| {
        Message::Scene(SceneCommand::EditLayerFistColour(
            layer_id,
            Color { r: value, g, b, a },
        ))
    })
    .step(0.01)
    .into();

    let green_slider: Element<'_, Message> = slider(0.0..=1.0, g, move |value| {
        Message::Scene(SceneCommand::EditLayerFistColour(
            layer_id,
            Color { r, g: value, b, a },
        ))
    })
    .step(0.01)
    .into();

    let blue_slider: Element<'_, Message> = slider(0.0..=1.0, b, move |value| {
        Message::Scene(SceneCommand::EditLayerFistColour(
            layer_id,
            Color { r, g, b: value, a },
        ))
    })
    .step(0.01)
    .into();

    let alpha_slider: Element<'_, Message> = slider(0.0..=1.0, a, move |value| {
        Message::Scene(SceneCommand::EditLayerFistColour(
            layer_id,
            Color { r, g, b, a: value },
        ))
    })
    .step(0.01)
    .into();

    column![
        colour_preview,
        column![
            row![text("R"), red_slider].spacing(16),
            row![text("G"), green_slider].spacing(16),
            row![text("B"), blue_slider].spacing(16),
            row![text("A"), alpha_slider].spacing(16)
        ]
    ]
    .spacing(8.0)
}
