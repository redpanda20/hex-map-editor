use iced::{
    Color, Element, Length,
    widget::{column, container, responsive, row, rule, slider, space, text},
};

use crate::{
    app::Message,
    state::{LayerMessage, Layers},
};

pub fn colour_panel<'a>(layers: &Layers) -> Element<'a, Message> {
    let active_layer = layers.get_active_layer();
    let active_colour = active_layer
        .map(|layer| layer.color)
        .unwrap_or(Color::BLACK);

    let square = responsive(|size| {
        let new_size = size.ratio(1.0);
        space().width(new_size.width).height(new_size.height).into()
    });

    let colour_preview = container(square)
        .height(Length::Shrink)
        .style(move |_theme| container::background(active_colour));

    let Color { r, g, b, a } = active_colour;

    let red_slider: Element<'_, Message> = slider(0.0..=1.0, r, move |value| {
        LayerMessage::ChangeActiveLayerColor(Color { r: value, g, b, a }).into()
    })
    .step(0.01)
    .into();

    let green_slider: Element<'_, Message> = slider(0.0..=1.0, g, move |value| {
        LayerMessage::ChangeActiveLayerColor(Color { r, g: value, b, a }).into()
    })
    .step(0.01)
    .into();

    let blue_slider: Element<'_, Message> = slider(0.0..=1.0, b, move |value| {
        LayerMessage::ChangeActiveLayerColor(Color { r, g, b: value, a }).into()
    })
    .step(0.01)
    .into();

    let alpha_slider: Element<'_, Message> = slider(0.0..=1.0, a, move |value| {
        LayerMessage::ChangeActiveLayerColor(Color { r, g, b, a: value }).into()
    })
    .step(0.01)
    .into();

    let controls = column![
        row![text("R"), red_slider].spacing(16),
        row![text("G"), green_slider].spacing(16),
        row![text("B"), blue_slider].spacing(16),
        row![text("A"), alpha_slider].spacing(16)
    ]
    .padding([0, 8]);

    let colour_editor = column![
        rule::horizontal(1),
        colour_preview,
        controls,
        space::vertical()
    ]
    .spacing(8)
    .padding(8);

    container(colour_editor)
        .style(container::bordered_box)
        .into()
}
