mod about;
mod canvas;
mod inspector;
mod keybinds;
mod layers;
mod panes;
mod toasts;
mod toolbar;

// Panes
pub use canvas::{CanvasEvent, canvas_panel};
pub use inspector::{Inspector, InspectorMessage};
pub use layers::{Layers, LayersMessage};
pub use panes::{Panes, PanesMessage};
pub use toolbar::{Toolbar, ToolbarMessage};

// Sub elements
pub use about::{About, AboutMessage};
pub use keybinds::{Binding, KeybindMessage, Keybinds};
pub use toasts::{ToastMessage, Toasts};

// Widgets
mod widget;
use widget::colour_picker;
