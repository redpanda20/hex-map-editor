use iced::{
    Element,
    widget::{PaneGrid, container, pane_grid},
};

use crate::{app::Message, domain::Scene, theme};

pub enum PaneKind {
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

    pub fn view<'a>(
        &'a self,
        scene: &'a Scene,
        inspector: impl Fn(&'a Scene) -> Element<'a, Message>,
        layers: impl Fn(&'a Scene) -> Element<'a, Message>,
    ) -> PaneGrid<'a, Message> {
        pane_grid(&self.state, |_id, state, _is_maximised| {
            let inner = match state {
                PaneKind::LayerStack => layers(scene),
                PaneKind::Inspector => inspector(scene),
            };

            pane_grid::Content::new(wrap_with_pane(inner))
        })
        .on_resize(10, |resize| {
            Message::Panes(PanesMessage::PaneResized(resize))
        })
        .spacing(4)
        .width(300)
    }
}

impl Default for Panes {
    fn default() -> Self {
        let layers_pane = pane_grid::Configuration::Pane(PaneKind::LayerStack);
        let inspector_pane = pane_grid::Configuration::Pane(PaneKind::Inspector);

        let config = pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Horizontal,
            ratio: 0.3,
            a: Box::new(inspector_pane),
            b: Box::new(layers_pane),
        };

        Self::new_with(config)
    }
}

pub fn wrap_with_pane<'a>(inner: Element<'a, Message>) -> Element<'a, Message> {
    container(inner).padding(8.0).style(theme::panel).into()
}
