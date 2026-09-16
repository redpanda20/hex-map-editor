use iced::{
    Element, Length, Padding, Task, alignment,
    mouse::Interaction,
    widget::{
        Button, Text, button, column, container, mouse_area, pick_list, row, rule, scrollable,
        space, text,
    },
};
use iced_fonts::lucide;

use crate::{
    app::{Action::SetLayer, Message},
    domain::{
        Layer, LayerInner, LayerKind, Scene,
        edit::{MoveLayerTo, PushLayer, RemoveLayer, Rename, SetVisible},
        id::LayerId,
    },
    ui::widgets::{context_menu, inline_text_field},
};

#[derive(Debug, Clone)]
pub enum LayersMessage {
    ChangeLayerType { kind: LayerKind },

    LayerEnter { layer: LayerId },
    LayerExit { layer: LayerId },

    DragLayerPick { picked: LayerId },
    DragLayerDropped { dropped: LayerId },
    DragLayerCancelled,
}

#[derive(Debug, Default)]
pub struct Layers {
    active_layer_type: LayerKind,
    dragged_layer: Option<LayerId>,
    hovered_layer: Option<LayerId>,
}

impl Layers {
    pub fn update(&mut self, message: LayersMessage) -> Task<Message> {
        match message {
            LayersMessage::ChangeLayerType { kind } => self.active_layer_type = kind,

            LayersMessage::LayerEnter { layer } => self.hovered_layer = Some(layer),
            LayersMessage::LayerExit { layer } => {
                if self.hovered_layer == Some(layer) {
                    self.hovered_layer = None
                }
            }

            LayersMessage::DragLayerPick { picked } => self.dragged_layer = Some(picked),
            LayersMessage::DragLayerCancelled => self.dragged_layer = None,
            LayersMessage::DragLayerDropped { dropped } => {
                if let Some(picked) = self.dragged_layer.take() {
                    self.hovered_layer = None;
                    return Task::done(
                        MoveLayerTo {
                            id: picked,
                            to: dropped,
                        }
                        .into(),
                    );
                }
            }
        }
        Task::none()
    }

    pub fn view<'a>(
        &self,
        scene: &'a Scene,
        active_layer: Option<LayerId>,
    ) -> Element<'a, Message> {
        // Draw a bar containing general layer info
        let content = column(scene.inner.iter().map(|layer| {
            layer_preview(layer, active_layer, self.dragged_layer, self.hovered_layer)
        }))
        .spacing(4.0)
        .height(Length::Fill);

        // Cancel an active dragging motion
        let content: Element<'_, Message> = match self.dragged_layer {
            None => content.into(),
            Some(_) => mouse_area(content)
                .on_release(Message::Layers(LayersMessage::DragLayerCancelled))
                .on_exit(Message::Layers(LayersMessage::DragLayerCancelled))
                .into(),
        };

        container(
            column![
                rule::horizontal(1),
                scrollable(content).height(Length::Fill),
                add_layer_button(self)
            ]
            .height(Length::Fill)
            .width(Length::Fill)
            .spacing(8.0)
            .padding(8.0),
        )
        .style(container::bordered_box)
        .into()
    }
}

fn layer_preview<'a>(
    layer: &'a Layer,
    active_layer: Option<LayerId>,
    dragged_layer: Option<LayerId>,
    hovered_layer: Option<LayerId>,
) -> Element<'a, Message> {
    let Layer {
        id,
        name,
        visible,
        kind,
    } = layer;

    let is_active = Some(*id) == active_layer;
    let is_dragged = Some(*id) == dragged_layer;
    let is_hovered = Some(*id) == hovered_layer;

    let content = container(
        row![
            if is_hovered {
                Some(drag_handle(id))
            } else {
                None
            },
            thumbnail(kind),
            inline_text_field(name, move |value| Rename {
                id: *id,
                name: value.to_string()
            }
            .into()),
            space::horizontal(),
            visible_toggle(id, visible),
        ]
        .align_y(alignment::Vertical::Center)
        .padding(Padding::from([4, 8]))
        .spacing(8),
    )
    .style(match (is_active, is_dragged) {
        (_, true) => container::secondary,
        (true, false) => container::rounded_box,
        (false, false) => container::transparent,
    });

    let row = mouse_area(content)
        .on_press(Message::Action(SetLayer(Some(*id))))
        .on_enter(Message::Layers(LayersMessage::LayerEnter { layer: *id }))
        .on_exit(Message::Layers(LayersMessage::LayerExit { layer: *id }))
        .on_release(Message::Layers(LayersMessage::DragLayerDropped {
            dropped: *id,
        }));

    context_menu(row, move || layer_context_menu(*id))
}

fn layer_context_menu<'a>(id: LayerId) -> Element<'a, Message> {
    container(
        column![
            column![text("Layer menu")].padding(8),
            rule::horizontal(1),
            column![
                // Duplicate layer,
                // Lock layer
                button(text("Delete layer"))
                    .width(Length::Fill)
                    .style(button::danger)
                    .on_press(RemoveLayer { id }.into())
            ]
            .padding(8)
        ]
        .spacing(0),
    )
    .style(container::bordered_box)
    .width(Length::Fixed(160.0))
    .into()
}

fn drag_handle<'a>(id: &LayerId) -> Element<'a, Message> {
    mouse_area(lucide::grip_vertical().style(text::secondary))
        .on_press(Message::Layers(LayersMessage::DragLayerPick {
            picked: *id,
        }))
        .interaction(Interaction::Grab)
        .into()
}

fn visible_toggle<'a>(id: &LayerId, visible: &bool) -> Button<'a, Message> {
    let inner = match visible {
        true => lucide::eye(),
        false => lucide::eye_off(),
    }
    .style(text::secondary);
    button(inner).style(button::text).padding(0).on_press(
        SetVisible {
            id: *id,
            visible: !*visible,
        }
        .into(),
    )
}

fn thumbnail<'a>(kind: &LayerInner) -> Text<'a> {
    match kind {
        LayerInner::Tiles(_) => lucide::grid_threexthree().style(text::primary),
        LayerInner::Perlin(_) => lucide::waves().style(text::secondary),
        LayerInner::Image(_) => lucide::image().style(text::secondary),
        LayerInner::Unknown(_) => lucide::message_circle_question().style(text::danger),
    }
}

fn add_layer_button<'a>(layers: &Layers) -> Element<'a, Message> {
    let add_layer_button = button(
        row![lucide::plus(), text("Add layer")]
            .spacing(4.0)
            .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .on_press(
        PushLayer {
            name: format!("{} layer", layers.active_layer_type),
            kind: layers.active_layer_type,
        }
        .into(),
    );

    let add_layer_list = pick_list(
        [LayerKind::Tiles, LayerKind::Noise, LayerKind::Image],
        Some(layers.active_layer_type),
        |kind| Message::Layers(LayersMessage::ChangeLayerType { kind }),
    );

    row![add_layer_button, add_layer_list].spacing(8.0).into()
}
