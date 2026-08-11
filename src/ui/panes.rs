use iced::{Element, widget::pane_grid};

use crate::{
    app::{App, Message},
    ui::{canvas_panel, inspector, inspector_panel, layer_stack_panel, toolbar_panel},
};

pub enum PaneKind {
    Canvas,
    Toolbar,
    Inspector,
    LayerStack,
}

#[derive(Debug, Clone, Copy)]
pub enum PanesMessage {
    PaneResized(pane_grid::ResizeEvent),
}

pub struct Panes {
    state: pane_grid::State<PaneKind>,
}

impl Panes {
    pub fn new() -> Self {
        let state = pane_grid::State::with_configuration(default_pane_config());

        Self { state }
    }

    pub fn update(&mut self, message: PanesMessage) {
        match message {
            PanesMessage::PaneResized(resize_event) => {
                let pane_grid::ResizeEvent { split, ratio } = resize_event;
                self.state.resize(split, ratio);
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        canvas: Element<'a, Message>,
        inspector: Element<'a, Message>,
    ) -> Element<'a, Message> {
        pane_grid(&self.state, |_id, state, _is_maximised| {
            let inner: Element<'_, Message> = match state {
                PaneKind::Toolbar => toolbar_panel(),
                PaneKind::LayerStack => layer_stack_panel(),
                PaneKind::Canvas => canvas,
                PaneKind::Inspector => inspector,
            };

            pane_grid::Content::new(inner)
        })
        .on_resize(10, |resize| {
            Message::Panes(PanesMessage::PaneResized(resize))
        })
        .spacing(2)
        .into()
    }
}

fn default_pane_config() -> pane_grid::Configuration<PaneKind> {
    let toolbar_pane = pane_grid::Configuration::Pane(PaneKind::Toolbar);
    let canvas_pane = pane_grid::Configuration::Pane(PaneKind::Canvas);
    let layers_pane = pane_grid::Configuration::Pane(PaneKind::LayerStack);
    let inspector_pane = pane_grid::Configuration::Pane(PaneKind::Inspector);

    let map_and_toolbar = pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 0.0,
        a: Box::new(toolbar_pane),
        b: Box::new(canvas_pane),
    };

    let layers_editor = pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Horizontal,
        ratio: 0.3,
        a: Box::new(inspector_pane),
        b: Box::new(layers_pane),
    };

    pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 0.8,
        a: Box::new(map_and_toolbar),
        b: Box::new(layers_editor),
    }
}
