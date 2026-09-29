//! Modal for choosing an export format and its settings.
//!
//! The dialog owns the settings state and reports confirmation to the caller;
//! it does not know how to export anything.

use std::ops::RangeInclusive;

use iced::{
    Alignment, Element, Length,
    widget::{button, column, container, opaque, pick_list, row, space, text},
};

use crate::{
    domain::print::{PageMargin, PageSize, PrintSettings, TileSize},
    infrastructure::ExportFormat,
    theme,
    ui::widgets::float_field,
};

/// Multiplier applied to the PNG's base resolution.
const PNG_SCALE_RANGE: RangeInclusive<f64> = 0.1..=4.0;
const TILE_SIZE_RANGE_CM: RangeInclusive<f64> = 0.5..=30.0;
const MARGIN_RANGE_CM: RangeInclusive<f64> = 0.0..=5.0;

const DIALOG_WIDTH: f32 = 380.0;
/// Shared by every dropdown, so they line up with the numeric fields.
const DROPDOWN_WIDTH: f32 = 100.0;

#[derive(Debug, Clone)]
pub enum ExportDialogMessage {
    Show,
    Hide,
    SetFormat(ExportFormat),
    SetPngScale(f64),
    SetPageSize(PageSize),
    SetTileSize(f64),
    SetMargin(f64),
    Confirm,
}

#[derive(Debug, Clone)]
pub struct ExportDialog {
    shown: bool,
    format: ExportFormat,
    png_scale: f64,
    page_size: PageSize,
    // Held as f64 (what the numeric field edits) so typed values round-trip
    // exactly; converted to f32 when building `PrintSettings`.
    tile_size_cm: f64,
    margin_cm: f64,
}

impl Default for ExportDialog {
    fn default() -> Self {
        let print = PrintSettings::DEFAULT;
        Self {
            shown: false,
            format: ExportFormat::Png,
            png_scale: 1.0,
            page_size: print.page_size,
            tile_size_cm: print.scale.cm as f64,
            margin_cm: print.margin.cm as f64,
        }
    }
}

impl ExportDialog {
    /// Multiplier for PNG exports. 1.0 is the current export resolution.
    pub fn png_scale(&self) -> f32 {
        self.png_scale as f32
    }

    /// Physical parameters for PDF exports.
    pub fn print_settings(&self) -> PrintSettings {
        PrintSettings {
            scale: TileSize {
                cm: self.tile_size_cm as f32,
            },
            page_size: self.page_size,
            margin: PageMargin {
                cm: self.margin_cm as f32,
            },
        }
    }

    /// Returns the format to export when the user confirms the dialog.
    pub fn update(&mut self, message: ExportDialogMessage) -> Option<ExportFormat> {
        match message {
            ExportDialogMessage::Show => self.shown = true,
            ExportDialogMessage::Hide => self.shown = false,
            ExportDialogMessage::SetFormat(format) => self.format = format,
            ExportDialogMessage::SetPngScale(value) => {
                set_clamped(&mut self.png_scale, value, PNG_SCALE_RANGE)
            }
            ExportDialogMessage::SetPageSize(page_size) => self.page_size = page_size,
            ExportDialogMessage::SetTileSize(value) => {
                set_clamped(&mut self.tile_size_cm, value, TILE_SIZE_RANGE_CM)
            }
            ExportDialogMessage::SetMargin(value) => {
                set_clamped(&mut self.margin_cm, value, MARGIN_RANGE_CM)
            }
            ExportDialogMessage::Confirm => {
                self.shown = false;
                return Some(self.format);
            }
        }

        None
    }

    pub fn view(&self) -> Element<'_, ExportDialogMessage> {
        if !self.shown {
            return space().into();
        }

        let format = labelled(
            "Format",
            pick_list(
                ExportFormat::ALL,
                Some(self.format),
                ExportDialogMessage::SetFormat,
            )
            .width(DROPDOWN_WIDTH)
            .style(theme::dropdown)
            .menu_style(theme::dropdown_menu),
        );

        let config = match self.format {
            ExportFormat::Png => self.png_config(),
            ExportFormat::Pdf => self.pdf_config(),
        };

        let buttons = row![
            space::horizontal(),
            button("Cancel")
                .style(theme::button_secondary)
                .on_press(ExportDialogMessage::Hide),
            button("Export")
                .style(theme::button_primary)
                .on_press(ExportDialogMessage::Confirm),
        ]
        .spacing(8);

        let content = column![
            text("Export scene").size(20),
            column![format, config].spacing(12),
            buttons
        ]
        .spacing(20);

        modal(content)
    }

    fn png_config(&self) -> Element<'_, ExportDialogMessage> {
        float_field("Scale", self.png_scale, ExportDialogMessage::SetPngScale).into()
    }

    fn pdf_config(&self) -> Element<'_, ExportDialogMessage> {
        let page_size = labelled(
            "Page size",
            pick_list(
                PageSize::ALL,
                Some(self.page_size),
                ExportDialogMessage::SetPageSize,
            )
            .width(DROPDOWN_WIDTH)
            .style(theme::dropdown)
            .menu_style(theme::dropdown_menu),
        );

        let tile_size = float_field(
            "Tile size (cm)",
            self.tile_size_cm,
            ExportDialogMessage::SetTileSize,
        );

        let margin = float_field("Margin (cm)", self.margin_cm, ExportDialogMessage::SetMargin);

        column![page_size, tile_size, margin].spacing(12).into()
    }
}

/// Numeric fields accept anything that parses as a float, including `inf` and `NaN`.
fn set_clamped(target: &mut f64, value: f64, range: RangeInclusive<f64>) {
    if value.is_finite() {
        *target = value.clamp(*range.start(), *range.end());
    }
}

/// A label on the left, a control on the right.
fn labelled<'a>(
    label: &'a str,
    control: impl Into<Element<'a, ExportDialogMessage>>,
) -> Element<'a, ExportDialogMessage> {
    row![
        text(label).style(text::secondary),
        space::horizontal(),
        control.into()
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn modal<'a>(
    content: impl Into<Element<'a, ExportDialogMessage>>,
) -> Element<'a, ExportDialogMessage> {
    let dialog = container(content)
        .padding(20)
        .width(DIALOG_WIDTH)
        .style(theme::modal);

    // `opaque` stops clicks reaching the editor underneath.
    opaque(
        container(dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(theme::backdrop),
    )
    .into()
}
