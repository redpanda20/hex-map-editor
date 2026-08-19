mod canvas;
mod inspector;
mod keybinds;
mod layers;
mod panes;
mod toasts;
mod toolbar;

pub use canvas::canvas_panel;
pub use inspector::{Inspector, InspectorMessage};
pub use layers::{Layers, LayersMessage};
pub use panes::{Panes, PanesMessage};
pub use toasts::{ToastMessage, Toasts};
pub use toolbar::{Toolbar, ToolbarMessage};

// Widgets
mod colour_picker;
use colour_picker::colour_picker;

pub use keybinds::Keybinds;
