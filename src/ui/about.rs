use iced::{
    Alignment, Element, Length,
    widget::{button, column, row, scrollable, space, text, text_input},
};

use crate::{app::Message, theme, ui::widgets::modal};

const LICENSE_NOTICES: &str = include_str!("../../license-notices.json");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, serde::Deserialize)]
struct License {
    #[allow(unused)]
    id: String,
    name: String,
    #[serde(rename = "text")]
    description: String,
    crates: Vec<Crate>,
}

#[derive(Debug, serde::Deserialize)]
struct Crate {
    name: String,
    version: String,
    repository: Option<String>,
}

fn get_license_notices() -> Vec<License> {
    let mut licenses: Vec<License> =
        serde_json::from_str(LICENSE_NOTICES).expect("Invalid license-notices.json");

    licenses.sort_by(|a, b| a.name.cmp(&b.name));

    licenses
}

#[derive(Debug, Clone)]
pub enum AboutMessage {
    Show,
    Hide,
}

pub struct About {
    licenses: Vec<License>,
    shown: bool,
}

impl Default for About {
    fn default() -> Self {
        Self::new()
    }
}

impl About {
    pub fn new() -> Self {
        let licenses = get_license_notices();
        Self {
            licenses,
            shown: false,
        }
    }

    pub fn update(&mut self, message: AboutMessage) {
        match message {
            AboutMessage::Show => self.shown = true,
            AboutMessage::Hide => self.shown = false,
        }
    }

    pub fn view(&self) -> Option<Element<'_, Message>> {
        if !self.shown {
            return None;
        }

        let version_info = column![
            text("Version").style(text::secondary),
            text!("{VERSION}").style(text::secondary),
        ];

        let licenses = column(self.licenses.iter().map(license_view))
            .spacing(16)
            .width(Length::Fill);

        let buttons = row![
            space::horizontal(),
            button("Close")
                .style(theme::button_secondary)
                .on_press(AboutMessage::Hide),
        ]
        .spacing(8);

        let content = column![
            text!("Hex Map Editor {VERSION}").size(24),
            version_info,
            scrollable(licenses).height(300),
            buttons
        ]
        .spacing(16)
        .align_x(Alignment::Center);

        Some(modal(content).map(Message::About))
    }
}

fn license_view<'a>(content: &'a License) -> Element<'a, AboutMessage> {
    let bold = iced::Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    };

    column![
        text(&content.name).size(18).font(bold),
        column(content.crates.iter().map(crates_view)).spacing(2),
        text(&content.description).size(12)
    ]
    .spacing(4)
    .into()
}

fn crates_view<'a>(content: &'a Crate) -> Element<'a, AboutMessage> {
    let repository: Element<'_, AboutMessage> = match &content.repository {
        None => space().into(),
        Some(repo) => text_input(repo, repo)
            .style(|theme, status| {
                let mut style = text_input::default(theme, status);
                style.border.width = 0.0;
                style
            })
            .into(),
    };
    row![
        text(&content.name),
        text(&content.version).style(text::secondary),
        repository
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

// fn modal<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
//     let close_button = row![
//         space::horizontal(),
//         button(lucide::x())
//             .style(button::text)
//             .on_press(Message::About(AboutMessage::Hide))
//     ];

//     let inner = column![close_button, container(content).padding(16)];

//     let modal = container(inner)
//         .height(Length::FillPortion(1))
//         .width(Length::Fixed(600.0))
//         .padding(0)
//         .style(container::rounded_box);

//     const BACKGROUND_COLOR: Color = {
//         let mut background = Color::BLACK;
//         background.a = 0.65;
//         background
//     };
//     container(modal)
//         .width(Length::Fill)
//         .height(Length::Fill)
//         .align_x(Alignment::Center)
//         .align_y(Alignment::Center)
//         .style(|_| container::background(BACKGROUND_COLOR))
//         .into()
// }
