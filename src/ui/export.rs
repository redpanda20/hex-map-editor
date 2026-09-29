//! Modal for choosing an export format and its settings.
//!
//! The dialog owns the settings state and reports confirmation to the caller;
//! it does not know how to export anything.

use iced::{
    Element, Length, Task,
    widget::{Column, button, column, pick_list, row, space, text},
};

use crate::{
    app::Message,
    domain::print::{PageSize, PrintSettings},
    infrastructure::{ExportFormat, ExportSettings, IoProcess},
    theme::{self, choice_marker},
    ui::widgets::{INPUT_WIDTH, f32_field, modal},
};

#[derive(Debug, Clone)]
pub enum ExportDialogMessage {
    Show,
    Hide,
    Confirm,

    SetFormat(ExportFormat),

    SetImageScale(f32),
    SetScale(f32),
    SetMargin(f32),
    SetPageSize(PageSize),
}

#[derive(Debug, Clone)]
pub struct ExportDialog {
    shown: bool,
    format: ExportFormat,

    image_scale: f32,
    settings: PrintSettings,
}

impl Default for ExportDialog {
    fn default() -> Self {
        Self {
            shown: false,
            format: ExportFormat::Pdf,
            image_scale: 100.0,
            settings: PrintSettings::DEFAULT,
        }
    }
}

impl ExportDialog {
    /// Returns the format to export when the user confirms the dialog.
    pub fn update(&mut self, message: ExportDialogMessage) -> Task<Message> {
        match message {
            ExportDialogMessage::Show => self.shown = true,
            ExportDialogMessage::Hide => self.shown = false,
            ExportDialogMessage::SetFormat(export_format) => self.format = export_format,

            ExportDialogMessage::SetImageScale(scale) => self.image_scale = scale,

            ExportDialogMessage::SetScale(scale) => self.settings.scale.cm = scale,
            ExportDialogMessage::SetMargin(margin) => self.settings.margin.cm = margin,
            ExportDialogMessage::SetPageSize(page) => self.settings.page_size = page,

            ExportDialogMessage::Confirm => {
                self.shown = false;

                let settings = match self.format {
                    ExportFormat::Png => ExportSettings::Png(self.image_scale),
                    ExportFormat::Pdf => ExportSettings::Pdf(self.settings),
                };
                return Task::done(Message::Export(settings, IoProcess::Start));
            }
        }

        Task::none()
    }

    pub fn view<'a>(&'a self) -> Option<Element<'a, Message>> {
        if !self.shown {
            return None;
        }

        let format_button = move |format: ExportFormat| {
            column![
                button(format.name())
                    .on_press_maybe(
                        (self.format != format).then_some(ExportDialogMessage::SetFormat(format)),
                    )
                    .style(theme::tool_button),
                choice_marker(self.format == format)
            ]
            .into()
        };

        let format_selector = row![
            space::horizontal(),
            row(ExportFormat::ALL.into_iter().map(format_button))
                .width(Length::Shrink)
                .spacing(16),
            space::horizontal()
        ];

        let config = match self.format {
            ExportFormat::Png => self.png_config(),
            ExportFormat::Pdf => self.pdf_config(),
        }
        .height(120);

        let is_valid = match self.format {
            ExportFormat::Png => self.image_scale > 0.0,
            ExportFormat::Pdf => self.settings.is_valid(),
        };

        let buttons = row![
            space::horizontal(),
            button("Cancel")
                .style(theme::button_secondary)
                .on_press(ExportDialogMessage::Hide),
            button("Export")
                .style(theme::button_primary)
                .on_press_maybe(is_valid.then_some(ExportDialogMessage::Confirm))
        ]
        .spacing(8);

        let content = column![
            text("Export scene").size(24),
            format_selector,
            config,
            buttons
        ]
        .spacing(16)
        .width(400);

        Some(modal(content).map(Message::ExportDialog))
    }

    fn png_config(&self) -> Column<'_, ExportDialogMessage> {
        column![f32_field(
            "Scale (pixels per hex)",
            self.image_scale,
            ExportDialogMessage::SetImageScale
        )]
    }

    fn pdf_config(&self) -> Column<'_, ExportDialogMessage> {
        let PrintSettings {
            scale,
            page_size,
            margin,
        } = self.settings;
        column![
            row![
                text("Page Size").style(text::secondary),
                space::horizontal(),
                pick_list(
                    PageSize::ALL,
                    Some(page_size),
                    ExportDialogMessage::SetPageSize
                )
                .style(theme::dropdown)
                .menu_style(theme::dropdown_menu)
                .width(INPUT_WIDTH)
            ],
            f32_field("Margin", margin.cm, ExportDialogMessage::SetMargin),
            f32_field(
                "Scale (cm per hex)",
                scale.cm,
                ExportDialogMessage::SetScale
            ),
        ]
        .spacing(8)
    }
}
