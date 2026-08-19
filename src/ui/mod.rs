mod canvas;
mod inspector;
mod keybinds;
mod layers;
mod panes;
mod toasts;
mod toolbar;

// Panes
pub use canvas::canvas_panel;
pub use inspector::{Inspector, InspectorMessage};
pub use layers::{Layers, LayersMessage};
pub use panes::{Panes, PanesMessage};
pub use toolbar::{Toolbar, ToolbarMessage};

// Sub elements
pub use keybinds::{Binding, KeybindAction, KeybindMessage, Keybinds};
pub use toasts::{ToastMessage, Toasts};

// Widgets
mod colour_picker;
use colour_picker::colour_picker;
