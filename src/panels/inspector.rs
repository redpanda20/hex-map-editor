use iced::{
    Color, Element, Length, alignment,
    widget::{
        Column, Row, button, column, container, responsive, row, slider, space, text, text_input,
    },
};
use iced_fonts::bootstrap;

use crate::{
    app::{EditorState, Message},
    state::{Layer, LayerMessage, Layers, SparseTiles},
};

pub fn inspector_panel<'a>(layers: &Layers, editor_state: &EditorState) -> Element<'a, Message> {
    if let Some(layer_id) = layers.active_layer {
        if let Some(Layer {
            name,
            visible,
            inner,
        }) = layers.inner.get(layer_id)
        {
            let starting_name = name;
            let name = &editor_state.active_layer_name;

            let content = match inner {
                crate::state::LayerInner::Tiles(sparse_tiles) => {
                    sparse_tile_details(layer_id, starting_name, name, visible, sparse_tiles)
                }
                crate::state::LayerInner::InvertedTiles(sparse_tiles) => {
                    sparse_tile_details(layer_id, starting_name, name, visible, sparse_tiles)
                }
            };

            return container(content)
                .style(container::bordered_box)
                .padding(8.0)
                .into();
        }
    }

    container(column![text("No active layer"), space::vertical()])
        .style(container::bordered_box)
        .padding(8.0)
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
        colour_panel(layer_id, tiles.colour)
    ]
}

fn name_input<'a>(
    layer_id: usize,
    starting_name: &String,
    name: &Option<String>,
) -> Row<'a, Message> {
    let title: Element<'a, Message> = if let Some(name) = name {
        text_input("Layer name...", &name)
            .on_input(|s| Message::LayerRename(Some(s)))
            .on_submit(Message::LayerRenameSubmit(layer_id))
            .into()
    } else {
        button(
            row![
                bootstrap::pencil_fill().style(text::secondary),
                text(starting_name.clone()).height(24.0)
            ]
            .spacing(4.0)
            .align_y(alignment::Vertical::Center),
        )
        .on_press(Message::LayerRenameStart(starting_name.clone()))
        .style(button::text)
        .into()
    };

    row![space::horizontal(), title, space::horizontal()].into()
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
        Message::LayerEvent(LayerMessage::EditLayerColour(
            layer_id,
            Color { r: value, g, b, a },
        ))
    })
    .step(0.01)
    .into();

    let green_slider: Element<'_, Message> = slider(0.0..=1.0, g, move |value| {
        Message::LayerEvent(LayerMessage::EditLayerColour(
            layer_id,
            Color { r, g: value, b, a },
        ))
    })
    .step(0.01)
    .into();

    let blue_slider: Element<'_, Message> = slider(0.0..=1.0, b, move |value| {
        Message::LayerEvent(LayerMessage::EditLayerColour(
            layer_id,
            Color { r, g, b: value, a },
        ))
    })
    .step(0.01)
    .into();

    let alpha_slider: Element<'_, Message> = slider(0.0..=1.0, a, move |value| {
        Message::LayerEvent(LayerMessage::EditLayerColour(
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
