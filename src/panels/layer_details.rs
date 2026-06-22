use iced::{
    Color, Element, Length,
    widget::{Column, Row, button, column, container, responsive, row, rule, slider, space, text},
};
use iced_fonts::bootstrap;

use crate::{
    app::Message,
    state::{LayerMessage, Layers},
};

pub fn layer_details<'a>(layers: &Layers) -> Element<'a, Message> {
    let (name, color, visible) = match layers.get_active_layer() {
        Some(layer) => (
            layer.name.clone(),
            layer.color.clone(),
            layer.visible.clone(),
        ),
        None => ("No layer selected".to_string(), Color::BLACK, false),
    };

    container(
        column![
            rule::horizontal(1),
            name_input(name),
            visible_toggle(visible),
            space::vertical(),
            colour_panel(color),
        ]
        .spacing(4.0),
    )
    .style(container::bordered_box)
    .padding(8.0)
    .into()
}

fn name_input<'a>(name: String) -> Row<'a, Message> {
    row![
        space::horizontal(),
        text(name).height(16.0),
        space::horizontal()
    ]
    .into()
}

fn visible_toggle<'a>(visible: bool) -> Row<'a, Message> {
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
        .on_press(Message::LayerEvent(
            LayerMessage::ChangeActiveLayerVisibility,
        ));
    row![space::horizontal(), toggle, space::horizontal()]
}

fn colour_panel<'a>(active_colour: Color) -> Column<'a, Message> {
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
