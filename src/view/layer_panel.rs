use iced::{
    Alignment, Element, Length, Task,
    widget::{
        button, checkbox, column, container, mouse_area, row, rule, scrollable, space, text,
        text_input,
    },
};
use iced_fonts::bootstrap;

use crate::{
    app::Message,
    state::{Layer, LayerMessage, Layers},
};

pub struct LayerPanel {
    pub edit_layer: Option<(usize, String)>,
}

#[derive(Debug, Clone)]
pub enum LayerPanelMessage {
    BeginLayerEdit(usize),
    LayerEdit(String),
    CommitLayerEdit,
}

impl From<LayerPanelMessage> for Message {
    fn from(value: LayerPanelMessage) -> Self {
        Message::LayerPanelEvent(value)
    }
}

impl LayerPanel {
    pub fn new() -> LayerPanel {
        LayerPanel { edit_layer: None }
    }

    pub fn update(&mut self, message: LayerPanelMessage) -> Task<Message> {
        match message {
            LayerPanelMessage::BeginLayerEdit(index) => {
                self.edit_layer = Some((index, String::new()));
            }
            LayerPanelMessage::LayerEdit(edit_name) => {
                if let Some((_index, name)) = self.edit_layer.as_mut() {
                    *name = edit_name;
                }
            }
            LayerPanelMessage::CommitLayerEdit => {
                if let Some((index, name)) = self.edit_layer.clone() {
                    self.edit_layer = None;
                    return Task::done(LayerMessage::ChangeLayerName(index, name).into());
                }
            }
        }

        Task::none()
    }
}

impl Default for LayerPanel {
    fn default() -> Self {
        Self::new()
    }
}

pub fn layer_panel<'a>(layer_panel: &LayerPanel, layers: &Layers) -> Element<'a, Message> {
    let layer_rows: Vec<Element<Message>> = layers
        .inner
        .iter()
        .enumerate()
        .map(|(i, layer)| layer_row(&layer_panel, &layer, i, layers.active_layer == Some(i)))
        .collect();

    let scrollable_content =
        scrollable(column(layer_rows).spacing(4.0).width(Length::Fill)).height(Length::Fill);

    let add_layer_button = button(row![bootstrap::plus_square(), text("Add layer")].spacing(16))
        .padding(8)
        .on_press(Message::LayerEvent(LayerMessage::AddLayer))
        .width(Length::Fill);

    let content = column![rule::horizontal(1), scrollable_content, add_layer_button]
        .height(Length::Fill)
        .width(Length::Fill)
        .spacing(8.0)
        .padding(8.0);

    container(content).style(container::bordered_box).into()
}

fn layer_row<'a>(
    layer_panel: &LayerPanel,
    layer: &Layer,
    layer_index: usize,
    is_active: bool,
) -> Element<'a, Message> {
    let is_editing = match layer_panel.edit_layer {
        Some((edit_index, _)) => edit_index == layer_index,
        None => false,
    };

    let visibility_toggle = checkbox(layer.visible).on_toggle(move |state| {
        Message::LayerEvent(LayerMessage::ChangeLayerVisibility(layer_index, state))
    });

    let name: Element<'_, LayerPanelMessage> = match (is_editing, is_active) {
        (true, ..) => text_input("Layer name...", &layer_panel.edit_layer.clone().unwrap().1)
            .width(Length::FillPortion(3))
            .on_input(LayerPanelMessage::LayerEdit)
            .on_submit(LayerPanelMessage::CommitLayerEdit)
            .into(),
        (false, true) => button(text(layer.name.clone()))
            .style(button::secondary)
            .on_press(LayerPanelMessage::BeginLayerEdit(layer_index))
            .into(),
        (false, false) => text(layer.name.clone()).into(),
    };

    let delete_button = button(bootstrap::trash())
        .style(button::danger)
        .on_press(Message::LayerEvent(LayerMessage::RemoveLayer(layer_index)));

    let content = row![
        visibility_toggle,
        name.map(|message| message.into()),
        space::horizontal(),
        delete_button
    ]
    .align_y(Alignment::Center)
    .spacing(16.0);

    let content = container(content)
        .padding([4.0, 8.0])
        .style(move |theme| match is_active {
            false => container::transparent(theme),
            true => container::background(theme.palette().primary.scale_alpha(0.2)),
        });

    mouse_area(content)
        .on_press(LayerMessage::ChangeActiveLayer(Some(layer_index)).into())
        .into()
}
