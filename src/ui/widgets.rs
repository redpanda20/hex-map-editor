mod bounded_float_field;
pub mod bounded_integer_field;
mod colour_field;
mod context_menu;
pub mod float_field;
mod text_field;

pub use bounded_float_field::bounded_float_field;
pub use bounded_integer_field::bounded_integer_field;
pub use colour_field::colour_field;
pub use context_menu::context_menu;
pub use float_field::float_field;
pub use text_field::inline_text_field;

// Local components
mod colour_picker;
use colour_picker::colour_picker;

/// Implements Into<iced::Length>
const INPUT_WIDTH: u32 = 80;
