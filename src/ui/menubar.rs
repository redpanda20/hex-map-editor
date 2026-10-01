use iced::{
    Alignment, Element, Length,
    widget::{button, column, container, row, rule, space, text},
};
use iced_fonts::lucide;

use crate::{
    app::{Action, Message},
    theme,
    ui::{Keybinds, widgets::popup_menu},
};

const MENU_WIDTH: f32 = 200.0;

pub struct Menubar;

impl Menubar {
    pub fn view<'a>(&self, keybinds: &'a Keybinds) -> Element<'a, Message> {
        let file_menu = button("File").style(theme::button_menu_trigger);
        let file_menu = popup_menu(file_menu, move || Self::file_menu(keybinds));

        let export = button("Export")
            .style(theme::button_primary)
            .on_press(Message::Action(Action::Export));

        let settings = button(lucide::settings()).style(theme::button_menu_trigger);
        let settings = popup_menu(settings, move || Self::settings_menu(keybinds));

        let content = row![file_menu, space::horizontal(), export, settings]
            .spacing(8)
            .padding(8)
            .align_y(Alignment::Center);

        container(content).style(theme::panel).into()
    }

    fn file_menu<'a>(keybinds: &Keybinds) -> Element<'a, Message> {
        menu_panel(
            column![
                menu_item("Open scene", Some(Action::Load), keybinds),
                menu_item("Save scene", Some(Action::Save), keybinds),
            ]
            .spacing(4),
        )
        .map(Message::Action)
    }

    fn settings_menu<'a>(keybinds: &Keybinds) -> Element<'a, Message> {
        menu_panel(
            column![
                // Intentional stub: no action yet.
                menu_item("Settings", None, keybinds),
                rule::horizontal(1),
                menu_item("About", Some(Action::About), keybinds),
            ]
            .spacing(4),
        )
        .map(Message::Action)
    }
}

/// Shared frame for menubar menus.
fn menu_panel<'a, M: 'a>(items: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(items)
        .padding(8)
        .style(theme::panel)
        .width(Length::Fixed(MENU_WIDTH))
        .into()
}

/// A full-width menu row: label on the left, optional shortcut hint on the right.
/// With no `action` the row is shown dimmed and does nothing.
fn menu_item<'a>(
    label: &'a str,
    action: Option<Action>,
    keybinds: &Keybinds,
) -> Element<'a, Action> {
    let shortcut = action
        .and_then(|action| keybinds.binding_for(action))
        .map(|binding| format!("{binding}"))
        .map(|s| text(s).size(12).style(text::secondary));

    let content = row![text(label).size(14), space::horizontal(), shortcut]
        .spacing(8)
        .align_y(Alignment::Center);

    button(content)
        .width(Length::Fill)
        .style(theme::button_menu_item)
        .on_press_maybe(action)
        .into()
}
