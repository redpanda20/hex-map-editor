mod canvas_panel;
mod layer_panel;
mod tool_panel;

use iced::widget::pane_grid;
use iced::widget::pane_grid::Configuration;

pub use canvas_panel::HexCanvas;

pub use tool_panel::toolbar_panel;

pub use layer_panel::LayerPanel;
pub use layer_panel::LayerPanelMessage;
pub use layer_panel::layer_panel;

pub enum PaneType {
    Canvas,
    Layers,
    Toolbar,
}

pub fn default_pane_config() -> Configuration<PaneType> {
    let toolbar_pane = pane_grid::Configuration::Pane(PaneType::Toolbar);
    let canvas_pane = pane_grid::Configuration::Pane(PaneType::Canvas);
    let layers_pane = pane_grid::Configuration::Pane(PaneType::Layers);

    let canvas_toolbar = Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 0.0,
        a: Box::new(toolbar_pane),
        b: Box::new(canvas_pane),
    };

    pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 0.8,
        a: Box::new(canvas_toolbar),
        b: Box::new(layers_pane),
    }
}
