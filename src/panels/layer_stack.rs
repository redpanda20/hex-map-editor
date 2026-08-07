use iced::{
    Element, Length, alignment,
    widget::{
        Button, button, column, container, mouse_area, pick_list, row, rule, scrollable, space,
        text,
    },
};
use iced_fonts::bootstrap;

use crate::{
    app::{EditorState, Message},
    state::{Layer, LayerInner, LayerMessage, LayerType, Layers, PerlinNoiseLayer, SparseTiles},
};

pub fn layer_stack_panel<'a>(
    layers: &'a Layers,
    editor_state: &EditorState,
) -> Element<'a, Message> {
    let active_layer = layers.active_layer;

    let content = scrollable(
        column(
            layers
                .inner
                .iter()
                .enumerate()
                .map(|(i, layer)| layer_preview(layer, i, Some(i) == active_layer)),
        )
        .spacing(4.0)
        .height(Length::Fill),
    )
    .height(Length::Fill);

    let add_layer_button = button(
        row![bootstrap::plus_lg(), text("Add layer")]
            .spacing(4.0)
            .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .on_press(Message::LayerEvent(LayerMessage::AddLayer(
        "Layer".to_string(),
        editor_state.active_layer_type,
    )));

    let add_layer_list = pick_list(
        [LayerType::Tiles, LayerType::PerlinNoise],
        Some(editor_state.active_layer_type),
        Message::ChangeLayerType,
    );

    let add_layer = row![add_layer_button, add_layer_list].spacing(8.0);

    container(
        column![rule::horizontal(1), content, add_layer]
            .height(Length::Fill)
            .width(Length::Fill)
            .spacing(8.0)
            .padding(8.0),
    )
    .style(container::bordered_box)
    .into()
}

fn layer_preview<'a>(
    layer: &'a Layer,
    layer_id: usize,
    is_active_layer: bool,
) -> Element<'a, Message> {
    let Layer {
        name,
        visible,
        inner,
    } = layer;

    let layer_preview = match inner {
        LayerInner::Tiles(sparse_tiles) => preview_tiles(sparse_tiles),
        LayerInner::InvertedTiles(sparse_tiles) => preview_invert_tiles(sparse_tiles),
        LayerInner::Perlin(perlin_noise_layer) => preview_noise(perlin_noise_layer),
    };

    let content = container(
        row![
            visible_toggle(layer_id, visible),
            layer_preview,
            text(name),
            space::horizontal(),
            delete_button(layer_id)
        ]
        .align_y(alignment::Vertical::Center)
        .spacing(8.0),
    )
    .style(match is_active_layer {
        true => container::rounded_box,
        false => container::transparent,
    });

    mouse_area(content)
        .on_press(Message::LayerEvent(LayerMessage::SetActiveLayer(Some(
            layer_id,
        ))))
        .into()
}

fn visible_toggle<'a>(layer_id: usize, visible: &bool) -> Button<'a, Message> {
    let inner = match visible {
        true => bootstrap::eye(),
        false => bootstrap::eye_slash(),
    }
    .style(text::secondary);
    button(inner)
        .style(button::text)
        .on_press(Message::LayerEvent(LayerMessage::EditLayerVisibility(
            layer_id, !visible,
        )))
}

fn preview_tiles<'a>(inner: &'a SparseTiles) -> Element<'a, Message> {
    let mut solid_colour = inner.get_colour();
    solid_colour.a = 1.0;

    match inner.is_empty() {
        true => bootstrap::hexagon(),
        false => bootstrap::hexagon_half(),
    }
    .color(solid_colour)
    .into()
}

fn preview_invert_tiles<'a>(inner: &'a SparseTiles) -> Element<'a, Message> {
    let mut solid_colour = inner.get_colour();
    solid_colour.a = 1.0;

    bootstrap::hexagon_fill().color(solid_colour).into()
}

fn preview_noise<'a>(_inner: &'a PerlinNoiseLayer) -> Element<'a, Message> {
    bootstrap::sliderstwo().into()
}

fn delete_button<'a>(layer_id: usize) -> Button<'a, Message> {
    button(bootstrap::trash_fill())
        .on_press(Message::LayerEvent(LayerMessage::RemoveLayer(layer_id)))
        .style(button::danger)
}
