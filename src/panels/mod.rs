mod canvas;
mod layer_details;
mod layers;
mod toasts;
mod tools;

use iced::widget::pane_grid;
use iced::widget::pane_grid::Configuration;

pub use layers::LayerEvent;
pub use layers::LayerManager;

pub use toasts::ToastEvent;
pub use toasts::ToastManager as Toasts;

pub use canvas::canvas_panel;
pub use layer_details::colour_panel as layer_details;
pub use layers::layer_panel;
pub use toasts::toast_widget;
pub use tools::toolbar_panel;

pub enum PaneType {
    Canvas,
    Layers,
    Toolbar,
    Colour,
}

pub fn default_pane_config() -> Configuration<PaneType> {
    let toolbar_pane = pane_grid::Configuration::Pane(PaneType::Toolbar);
    let canvas_pane = pane_grid::Configuration::Pane(PaneType::Canvas);
    let layers_pane = pane_grid::Configuration::Pane(PaneType::Layers);
    let colour_pane = pane_grid::Configuration::Pane(PaneType::Colour);

    let map_and_toolbar = Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 0.0,
        a: Box::new(toolbar_pane),
        b: Box::new(canvas_pane),
    };

    let layers_editor = Configuration::Split {
        axis: pane_grid::Axis::Horizontal,
        ratio: 0.3,
        a: Box::new(colour_pane),
        b: Box::new(layers_pane),
    };

    pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 0.8,
        a: Box::new(map_and_toolbar),
        b: Box::new(layers_editor),
    }
}
