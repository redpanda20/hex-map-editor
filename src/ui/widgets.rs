mod bounded_float_field;
mod bounded_integer_field;
mod colour_field;
mod context_menu;
mod numeric_field;
mod text_field;

pub use bounded_float_field::bounded_float_field;
pub use bounded_integer_field::bounded_integer_field;
pub use colour_field::colour_field;
pub use context_menu::{context_menu, popup_menu};
pub use modal::modal;
pub use numeric_field::{f32_field, float_field, integer_field};
pub use text_field::inline_text_field;

// Local components
mod colour_picker;
mod helper;
use colour_picker::colour_picker;

/// Implements Into<iced::Length>
pub const INPUT_WIDTH: u32 = 80;

mod modal {
    use crate::theme;

    use iced::{
        Alignment, Element, Length,
        widget::{container, opaque},
    };

    pub fn modal<'a, Message: 'a>(
        content: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let dialog = container(content)
            .padding(16)
            .max_width(600)
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
    }
}
