use std::collections::BTreeMap;

use iced::{
    Alignment, Element, Length, Task, clipboard,
    widget::{button, column, container, row, rule, scrollable, space, text, tooltip},
};
use iced_fonts::lucide;

use crate::{app::Message, theme, ui::widgets::modal};

const LICENSE_NOTICES: &str = include_str!("../../license-notices.json");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_license_notices() -> Vec<License> {
    let raw: Vec<RawLicense> =
        serde_json::from_str(LICENSE_NOTICES).expect("Invalid license-notices.json");

    let mut grouped: BTreeMap<String, License> = BTreeMap::new();

    for RawLicense {
        id,
        name,
        content,
        crates,
    } in raw
    {
        let entry = grouped.entry(id).or_insert_with(|| License {
            name,
            notices: Vec::new(),
        });

        entry.notices.push(Notice {
            works: crates,
            content,
        });
    }

    grouped.into_values().collect()
}

#[derive(Debug, serde::Deserialize)]
struct RawLicense {
    id: String,
    name: String,
    #[serde(rename = "text")]
    content: String,
    crates: Vec<Crate>,
}

// <project-name> <version> <option><repository link>
#[derive(Debug, serde::Deserialize)]
struct Crate {
    name: String,
    version: String,
    repository: Option<String>,
}

struct License {
    name: String,
    notices: Vec<Notice>,
}

struct Notice {
    works: Vec<Crate>,
    content: String,
}

#[derive(Debug, Clone)]
pub enum AboutMessage {
    Show,
    Hide,
    Select { license: usize },
    Toggle { notice: usize },
    CopyText(String),
}

#[derive(Default)]
pub struct About {
    loaded: Option<Loaded>,
    shown: bool,
}

struct Loaded {
    licenses: Vec<License>,
    selected: usize,
    expanded: Option<usize>,
}

impl Loaded {
    fn load() -> Self {
        Self {
            licenses: get_license_notices(),
            selected: 0,
            expanded: None,
        }
    }
}

impl About {
    pub fn update(&mut self, message: AboutMessage) -> Task<Message> {
        match message {
            AboutMessage::Show => {
                if self.loaded.is_none() {
                    self.loaded = Some(Loaded::load());
                }
                self.shown = true;
            }
            AboutMessage::Hide => self.shown = false,
            AboutMessage::Select { license } => {
                if let Some(loaded) = &mut self.loaded
                    && license < loaded.licenses.len()
                {
                    loaded.selected = license;
                    loaded.expanded = None;
                    if loaded.licenses[license].notices.len() == 1 {
                        loaded.expanded = Some(0)
                    }
                }
            }
            AboutMessage::Toggle { notice } => {
                if let Some(loaded) = &mut self.loaded {
                    loaded.expanded = match loaded.expanded {
                        Some(i) if i == notice => None,
                        _ => Some(notice),
                    };
                }
            }
            AboutMessage::CopyText(text) => {
                return clipboard::write(text);
            }
        }
        Task::none()
    }

    pub fn view<'a>(&'a self) -> Option<Element<'a, AboutMessage>> {
        if !self.shown {
            return None;
        }
        let Some(loaded) = &self.loaded else {
            return None;
        };

        let overview = loaded
            .licenses
            .iter()
            .enumerate()
            .map(|(i, l)| Self::license_preview(l, i, loaded.selected));

        let current_license = loaded.licenses.get(loaded.selected);
        let details = match current_license {
            Some(license) => column(
                license
                    .notices
                    .iter()
                    .enumerate()
                    .map(|(i, n)| Self::notice_details(n, i, loaded.expanded)),
            ),
            // This should be impossible. But crashing is suboptimal
            None => column![text("You encountered an error")],
        };

        let header = column![
            text("Arena").size(24),
            text("Tactical Encounter Builder").style(text::secondary),
            text!("Version {VERSION}").style(text::secondary)
        ]
        .spacing(4)
        .align_x(Alignment::Center)
        .width(Length::Fill);

        let content = row![
            scrollable(column(overview).spacing(8).width(Length::FillPortion(1))),
            scrollable(details.spacing(16).width(Length::FillPortion(2))).spacing(4)
        ]
        .spacing(16)
        .height(800);

        let buttons = row![
            space::horizontal(),
            button("Close")
                .style(theme::button_secondary)
                .on_press(AboutMessage::Hide),
        ];

        let content = column![header, content, buttons].spacing(16);

        Some(modal(content))
    }

    fn license_preview<'a>(
        license: &'a License,
        index: usize,
        selected: usize,
    ) -> Element<'a, AboutMessage> {
        let crate_count = license.notices.len();
        let inner = column![
            text(&license.name),
            text!("{crate_count} crates")
                .size(12)
                .style(text::secondary)
        ]
        .width(Length::Fill)
        .spacing(4);

        button(inner)
            .on_press(AboutMessage::Select { license: index })
            .style(if index == selected {
                theme::button_primary
            } else {
                theme::button_ghost
            })
            .into()
    }

    fn notice_details<'a>(
        notice: &'a Notice,
        index: usize,
        expanded: Option<usize>,
    ) -> Element<'a, AboutMessage> {
        let is_expanded = Some(index) == expanded;

        let works = column(notice.works.iter().map(Self::crate_details));
        let notice = text(&notice.content).size(12);

        let accordion_icon = if is_expanded {
            lucide::chevron_up()
        } else {
            lucide::chevron_right()
        };
        let accordion_header = row![text("License"), accordion_icon]
            .align_y(Alignment::Center)
            .spacing(8);
        let accordion_control = button(accordion_header)
            .style(if is_expanded {
                theme::button_secondary
            } else {
                theme::button_ghost
            })
            .on_press(AboutMessage::Toggle { notice: index });

        column![
            accordion_control,
            works.spacing(4),
            is_expanded.then_some(notice),
            rule::horizontal(2).style(rule::weak)
        ]
        .width(Length::FillPortion(2))
        .spacing(4)
        .into()
    }

    fn crate_details<'a>(crate_info: &'a Crate) -> Element<'a, AboutMessage> {
        let Crate {
            name,
            version,
            repository,
        } = &crate_info;

        row![
            text(name),
            text(version).style(text::secondary),
            space::horizontal(),
            repository.as_deref().map(link)
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    }
}

fn link<'a>(value: &'a str) -> Element<'a, AboutMessage> {
    let content = button(lucide::copy().style(text::secondary))
        .style(theme::button_ghost)
        .on_press(AboutMessage::CopyText(value.to_string()));
    let on_hover = container(text(value)).padding(8).style(theme::panel);

    tooltip(content, on_hover, tooltip::Position::Bottom).into()
}
