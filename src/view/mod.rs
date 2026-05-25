mod canvas_panel;
mod colour_panel;
mod layer_panel;
mod tool_panel;

use iced::widget::pane_grid;
use iced::widget::pane_grid::Configuration;

pub use canvas_panel::HexCanvas;

pub use colour_panel::colour_panel;
pub use tool_panel::toolbar_panel;

pub use layer_panel::LayerPanel;
pub use layer_panel::LayerPanelMessage;
pub use layer_panel::layer_panel;

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
