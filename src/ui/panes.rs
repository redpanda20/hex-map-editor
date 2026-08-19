use iced::{Element, widget::pane_grid};

use crate::{app::Message, domain::Scene};

pub enum PaneKind {
    Canvas,
    Toolbar,
    Inspector,
    LayerStack,
}

#[derive(Debug, Clone)]
pub enum PanesMessage {
    PaneResized(pane_grid::ResizeEvent),
}

pub struct Panes {
    state: pane_grid::State<PaneKind>,
}

impl Panes {
    pub fn new_with(config: impl Into<pane_grid::Configuration<PaneKind>>) -> Self {
        let state = pane_grid::State::with_configuration(config);

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

    // view: impl Fn(&'a App) -> Element<'a, Message>
    pub fn view<'a>(
        &'a self,
        scene: &'a Scene,
        canvas: impl Fn(&'a Scene) -> Element<'a, Message>,
        inspector: impl Fn(&'a Scene) -> Element<'a, Message>,
        layers: impl Fn(&'a Scene) -> Element<'a, Message>,
        toolbar: impl Fn(&'a Scene) -> Element<'a, Message>,
    ) -> Element<'a, Message> {
        pane_grid(&self.state, |_id, state, _is_maximised| {
            let inner: Element<'_, Message> = match state {
                PaneKind::Toolbar => toolbar(scene),
                PaneKind::LayerStack => layers(scene),
                PaneKind::Canvas => canvas(scene),
                PaneKind::Inspector => inspector(scene),
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

impl Default for Panes {
    fn default() -> Self {
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

        let config = pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: 0.8,
            a: Box::new(map_and_toolbar),
            b: Box::new(layers_editor),
        };

        Self::new_with(config)
    }
}
