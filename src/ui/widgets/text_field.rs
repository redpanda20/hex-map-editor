use iced::{
    Element, Length, Theme,
    widget::{button, column, container, space, text, text_input},
};

use crate::ui::widgets::Property;
use crate::{app::Message, ui::InspectorMessage};

pub fn text_field<'a>(
    property: &'a Property<String>,
    starting_value: &'a str,
    on_submit: impl Fn(&str) -> Message,
) -> Element<'a, Message> {
    let Some(value) = &property.value() else {
        return inline_text_element(property, starting_value);
    };

    text_input("", value)
        .on_input(move |value| {
            property.set(value);
            Message::Inspector(InspectorMessage::Changed)
        })
        .on_submit((on_submit)(value))
        .style(|theme, status| {
            match status {
                text_input::Status::Focused { is_hovered: true } => (),
                _ => property.clear(),
            }

            text_input::default(theme, status)
        })
        .into()
}

fn inline_text_element<'a>(
    property: &'a Property<String>,
    starting_value: &'a str,
) -> Element<'a, Message> {
    let text_content = column![
        // 19.0 Happens to be the height that matches text_input
        text(starting_value).height(19.0),
        container(space())
            .width(Length::Fill)
            .height(2)
            .style(|theme: &Theme| {
                container::Style {
                    background: Some(theme.palette().primary.into()),
                    ..Default::default()
                }
            })
    ]
    .width(Length::Shrink);
    button(text_content)
        .on_press_with(move || {
            property.set(starting_value.to_string());
            Message::Inspector(InspectorMessage::Changed)
        })
        .style(button::text)
        .into()
}
