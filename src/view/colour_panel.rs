use iced::{
    Color, Element, Length,
    widget::{column, container, responsive, row, slider, space, text},
};

use crate::{app::Message, state::Layer, view::LayerPanel};

pub fn colour_panel<'a>(layer_panel: &LayerPanel, layers: &Vec<Layer>) -> Element<'a, Message> {
    let active_colour = match layer_panel.active_layer.and_then(|index| layers.get(index)) {
        Some(layer) => layer.color,
        None => Color::BLACK,
    };

    let square = responsive(|size| {
        let new_size = size.ratio(1.0);
        space().width(new_size.width).height(new_size.height).into()
    });

    let colour_preview = container(square)
        .height(Length::Shrink)
        .style(move |_theme| container::background(active_colour));

    let mut controls = column![];

    if let Some(active_layer_index) = layer_panel.active_layer {
        let Color { r, g, b, a } = active_colour;

        let red_slider: Element<'_, Message> = slider(0.0..=1.0, r, move |value| {
            Message::EditLayerColor(active_layer_index, Color { r: value, g, b, a })
        })
        .step(0.01)
        .into();

        let green_slider: Element<'_, Message> = slider(0.0..=1.0, g, move |value| {
            Message::EditLayerColor(active_layer_index, Color { r, g: value, b, a })
        })
        .step(0.01)
        .into();

        let blue_slider: Element<'_, Message> = slider(0.0..=1.0, b, move |value| {
            Message::EditLayerColor(active_layer_index, Color { r, g, b: value, a })
        })
        .step(0.01)
        .into();

        let alpha_slider: Element<'_, Message> = slider(0.0..=1.0, a, move |value| {
            Message::EditLayerColor(active_layer_index, Color { r, g, b, a: value })
        })
        .step(0.01)
        .into();

        controls = controls.push(row![text("R"), red_slider].spacing(16));
        controls = controls.push(row![text("G"), green_slider].spacing(16));
        controls = controls.push(row![text("B"), blue_slider].spacing(16));
        controls = controls.push(row![text("A"), alpha_slider].spacing(16));
    }

    let colour_editor = column![colour_preview, controls, space::vertical()]
        .spacing(16)
        .padding(8);

    container(colour_editor)
        .style(container::bordered_box)
        .into()
}
