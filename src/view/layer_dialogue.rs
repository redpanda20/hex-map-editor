use iced::{
    Alignment, Border, Color, Element, Length, Task,
    border::Radius,
    widget::{
        button, checkbox, column, container, mouse_area, row, rule, scrollable, space, text,
        text_input,
    },
};

use crate::{app::Message, state::Layer};

pub struct LayerPanel {
    pub active_layer: Option<usize>,

    pub edit_layer: Option<(usize, String)>,
}

#[derive(Debug, Clone)]
pub enum LayerPanelMessage {
    SelectLayer(Option<usize>),

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
    pub fn update(&mut self, message: LayerPanelMessage) -> Task<Message> {
        match message {
            LayerPanelMessage::SelectLayer(optional_index) => {
                if self.active_layer != optional_index {
                    self.edit_layer = None;
                }
                self.active_layer = optional_index
            }
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
                    return Task::done(Message::EditLayerName(index, name));
                }
            }
        }

        Task::none()
    }
}

pub fn layer_panel<'a>(layer_panel: &LayerPanel, layers: &Vec<Layer>) -> Element<'a, Message> {
    let layer_rows: Vec<Element<Message>> = layers
        .iter()
        .enumerate()
        .map(|(i, layer)| layer_row(&layer_panel, &layer, i))
        .collect();

    let scrollable_content =
        scrollable(column(layer_rows).spacing(2.0).width(Length::Fill)).height(Length::Fill);

    let add_layer_button = button(text("Add Layer").size(16))
        .on_press(Message::AddLayer)
        .width(Length::Fill);

    let heading = text("Layers")
        .width(Length::Fill)
        .align_x(Alignment::Center);

    column![
        rule::horizontal(2.0),
        heading,
        scrollable_content,
        add_layer_button
    ]
    .width(220)
    .height(Length::Fill)
    .spacing(4.0)
    .padding(4.0)
    .into()
}

fn layer_row<'a>(
    layer_panel: &LayerPanel,
    layer: &Layer,
    layer_index: usize,
) -> Element<'a, Message> {
    let is_active = layer_panel.active_layer == Some(layer_index);
    let is_editing = match layer_panel.edit_layer {
        Some((edit_index, _)) => edit_index == layer_index,
        None => false,
    };

    let visibility_toggle = checkbox(layer.visible)
        .on_toggle(move |state| Message::EditLayerVisibility(layer_index, state));

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

    let delete_button = button(text("X"))
        .style(button::danger)
        .on_press(Message::RemoveLayer(layer_index));

    let content = row![
        visibility_toggle,
        name.map(|message| message.into()),
        space::horizontal(),
        delete_button
    ]
    .align_y(Alignment::Center)
    .spacing(6.0);

    let content = container(content)
        .padding(4.0)
        .style(move |_| match is_active {
            true => container::Style {
                background: Some(iced::Background::Color(Color::from_rgba8(
                    80, 120, 200, 0.2,
                ))),
                border: Border {
                    color: Color::BLACK,
                    width: 0.0,
                    radius: Radius::new(2.0),
                },
                ..Default::default()
            },
            false => container::Style::default(),
        });

    mouse_area(content)
        .on_press(LayerPanelMessage::SelectLayer(Some(layer_index)).into())
        .into()
}
