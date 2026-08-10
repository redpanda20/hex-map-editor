use iced::{
    Color, Element, Length, alignment, padding,
    widget::{
        Column, Row, button, column, container, responsive, row, rule, slider, space, text,
        text_input,
    },
};
use iced_fonts::bootstrap;

use crate::{
    app::{EditorState, Message},
    state::{Layer, LayerMessage, Layers, PerlinNoiseLayer, SparseTiles},
};

pub fn inspector_panel<'a>(layers: &Layers, editor_state: &EditorState) -> Element<'a, Message> {
    // Check for active content
    let Some((layer_id, layer)) = layers
        .active_layer
        .and_then(|id| layers.inner.get(id).map(|layer| (id, layer)))
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
    let name = &editor_state.active_layer_name;

    let content = match inner {
        crate::state::LayerInner::Tiles(sparse_tiles) => {
            sparse_tile_details(layer_id, starting_name, name, visible, sparse_tiles)
        }
        crate::state::LayerInner::InvertedTiles(sparse_tiles) => {
            sparse_tile_details(layer_id, starting_name, name, visible, sparse_tiles)
        }
        crate::state::LayerInner::Perlin(content) => {
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

fn sparse_tile_details<'a>(
    layer_id: usize,
    starting_name: &String,
    name: &Option<String>,
    visible: &bool,
    tiles: &SparseTiles,
) -> Column<'a, Message> {
    column![
        name_input(layer_id, starting_name, name),
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
                Message::LayerEvent(LayerMessage::EditLayerSeed(layer_id, new_seed))
            }),
        text(seed).style(text::secondary)
    ]
    .spacing(4.0)
    .align_y(alignment::Vertical::Center);

    let inverse_scale = 1.0 / scale;
    let scale_controls = row![
        text!("{inverse_scale:.2}").style(text::secondary),
        slider(1.0..=20.0, inverse_scale, move |value| {
            Message::LayerEvent(LayerMessage::EditLayerScale(layer_id, 1.0 / value))
        })
    ]
    .spacing(8.0)
    .align_y(alignment::Vertical::Center);

    let threshold_controls = row![
        text!("{threshold:.2} / 1.00").style(text::secondary),
        slider(0.0..=100.0, threshold * 100.0, move |value| {
            Message::LayerEvent(LayerMessage::EditLayerThreshold(layer_id, value / 100.0))
        })
    ]
    .spacing(8.0)
    .align_y(alignment::Vertical::Center);

    let (octave_count, persistence) = match octaves {
        crate::state::NoiseOctaves::One => (1, 0.0),
        crate::state::NoiseOctaves::Many { count, persistence } => (*count as i32, *persistence),
    };
    let mut octave_controls = column![];

    octave_controls = octave_controls.push(text("Count:"));
    octave_controls = octave_controls.push(
        row![
            text!("{octave_count}").style(text::secondary),
            slider(1..=8, octave_count, move |value| {
                Message::LayerEvent(LayerMessage::EditLayerOctaves(layer_id, value as usize))
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
                slider(1.0..=10.0, persistence * 10.0, move |value| {
                    Message::LayerEvent(LayerMessage::EditLayerPersistence(layer_id, value / 10.0))
                })
            ]
            .spacing(8.0)
            .align_y(alignment::Vertical::Center),
        );
    }

    column![
        name_input(layer_id, starting_name, name),
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
) -> Row<'a, Message> {
    if let Some(name) = name {
        row![
            bootstrap::input_cursor().style(text::secondary),
            text_input("Layer name...", &name)
                .on_input(|s| Message::LayerRename(Some(s)))
                .on_submit(Message::LayerRenameSubmit(layer_id))
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
            .on_press(Message::LayerRenameStart(starting_name.clone()))
            .style(button::text),
            space::horizontal()
        ]
    }
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
    let toggle = button(inner)
        .style(button::text)
        .on_press(Message::LayerEvent(LayerMessage::EditLayerVisibility(
            layer_id, !visible,
        )));
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
        Message::LayerEvent(LayerMessage::EditLayerFistColour(
            layer_id,
            Color { r: value, g, b, a },
        ))
    })
    .step(0.01)
    .into();

    let green_slider: Element<'_, Message> = slider(0.0..=1.0, g, move |value| {
        Message::LayerEvent(LayerMessage::EditLayerFistColour(
            layer_id,
            Color { r, g: value, b, a },
        ))
    })
    .step(0.01)
    .into();

    let blue_slider: Element<'_, Message> = slider(0.0..=1.0, b, move |value| {
        Message::LayerEvent(LayerMessage::EditLayerFistColour(
            layer_id,
            Color { r, g, b: value, a },
        ))
    })
    .step(0.01)
    .into();

    let alpha_slider: Element<'_, Message> = slider(0.0..=1.0, a, move |value| {
        Message::LayerEvent(LayerMessage::EditLayerFistColour(
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
