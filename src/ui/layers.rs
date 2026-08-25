use iced::{
    Color, Element, Length, Task, alignment,
    widget::{
        Button, Text, button, column, container, mouse_area, pick_list, row, rule, scrollable,
        space, text,
    },
};
use iced_fonts::bootstrap;

use crate::{
    app::{Action::SetLayer, Message},
    domain::{
        Layer, LayerInner, LayerKind, Scene,
        edit::{PushLayer, RemoveLayer, SetVisible, SwapLayers},
        id::LayerId,
    },
};

#[derive(Debug, Clone)]
pub enum LayersMessage {
    ChangeLayerType { kind: LayerKind },
    DragLayerPick { picked: LayerId },
    DragLayerDropped { dropped: LayerId },
    DragLayerCancelled,
}

#[derive(Debug, Default)]
pub struct Layers {
    active_layer_type: LayerKind,
    dragged_layer: Option<LayerId>,
}

impl Layers {
    pub fn update(&mut self, message: LayersMessage) -> Task<Message> {
        match message {
            LayersMessage::ChangeLayerType { kind } => self.active_layer_type = kind,
            LayersMessage::DragLayerPick { picked } => self.dragged_layer = Some(picked),
            LayersMessage::DragLayerCancelled => self.dragged_layer = None,
            LayersMessage::DragLayerDropped { dropped } => {
                if let Some(picked) = self.dragged_layer.take() {
                    let swap = SwapLayers {
                        id: picked,
                        to: dropped,
                    };

                    return Task::done(Message::Scene(Box::new(swap)));
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
        let content = column(
            scene
                .inner
                .iter()
                .map(|layer| layer_preview(layer, active_layer, self.dragged_layer)),
        )
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
) -> Element<'a, Message> {
    let Layer {
        id,
        name,
        visible,
        kind,
    } = layer;

    let is_active = Some(*id) == active_layer;
    let is_dragged = Some(*id) == dragged_layer;

    let content = container(
        row![
            drag_handle(id),
            visible_toggle(id, visible),
            thumbnail(kind),
            text(name),
            space::horizontal(),
            delete_button(id)
        ]
        .align_y(alignment::Vertical::Center)
        .spacing(8),
    )
    .style(match (is_active, is_dragged) {
        (_, true) => container::secondary,
        (true, false) => container::rounded_box,
        (false, false) => container::transparent,
    });

    mouse_area(content)
        .on_press(Message::Action(SetLayer(Some(*id))))
        .on_release(Message::Layers(LayersMessage::DragLayerDropped {
            dropped: *id,
        }))
        .into()
}

fn drag_handle<'a>(id: &LayerId) -> Element<'a, Message> {
    mouse_area(bootstrap::grip_vertical().style(text::secondary))
        .on_press(Message::Layers(LayersMessage::DragLayerPick {
            picked: *id,
        }))
        .into()
}

fn visible_toggle<'a>(id: &LayerId, visible: &bool) -> Button<'a, Message> {
    let inner = match visible {
        true => bootstrap::eye(),
        false => bootstrap::eye_slash(),
    }
    .style(text::secondary);
    button(inner)
        .style(button::text)
        .padding(0)
        .on_press(Message::Scene(Box::new(SetVisible {
            id: *id,
            visible: !*visible,
        })))
}

fn delete_button<'a>(id: &LayerId) -> Button<'a, Message> {
    button(bootstrap::trash_fill())
        .on_press(Message::Scene(Box::new(RemoveLayer { id: *id })))
        .style(button::danger)
}

fn thumbnail<'a>(kind: &LayerInner) -> Text<'a> {
    match kind {
        LayerInner::Tiles(tiles) => match (tiles.is_empty(), tiles.is_inverted()) {
            (_, true) => bootstrap::hexagon_fill(),
            (true, _) => bootstrap::hexagon(),
            (false, _) => bootstrap::hexagon(),
        }
        .color(tiles.colour.opaque()),
        LayerInner::Perlin(_) => bootstrap::sliderstwo(),
    }
}

trait ColorExt {
    fn opaque(self) -> Self;
}

impl ColorExt for Color {
    fn opaque(self) -> Self {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: 1.0,
        }
    }
}

fn add_layer_button<'a>(layers: &Layers) -> Element<'a, Message> {
    let add_layer_button = button(
        row![bootstrap::plus_lg(), text("Add layer")]
            .spacing(4.0)
            .align_y(alignment::Vertical::Center),
    )
    .width(Length::Fill)
    .on_press(Message::Scene(Box::new(PushLayer {
        name: "New layer".to_string(),
        kind: layers.active_layer_type,
    })));

    let add_layer_list = pick_list(
        [LayerKind::Tiles, LayerKind::Noise],
        Some(layers.active_layer_type),
        |kind| Message::Layers(LayersMessage::ChangeLayerType { kind }),
    );

    row![add_layer_button, add_layer_list].spacing(8.0).into()
}
