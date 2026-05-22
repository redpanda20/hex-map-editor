use iced::{
    Element, Length,
    widget::{button, column, radio, row, space, text},
};

use crate::{
    app::Message,
    view::{CanvasSettings, Tool},
};

pub fn toolbar_dialogue(settings: &CanvasSettings) -> Element<'_, Message> {
    let paint_tool = radio("Paint", Tool::Paint, Some(settings.tool), |_| {
        Message::ChangeTool(Tool::Paint)
    });
    let pan_tool = radio("Pan", Tool::Pan, Some(settings.tool), |_| {
        Message::ChangeTool(Tool::Pan)
    });
    let erase_tool = radio("Erase", Tool::Erase, Some(settings.tool), |_| {
        Message::ChangeTool(Tool::Erase)
    });

    let export_png = button(text("Export PNG"))
        .on_press(Message::ExportPng)
        .style(iced::widget::button::secondary);

    let toolbar = row![paint_tool, pan_tool, erase_tool].spacing(8.0);

    let export_tools = row![export_png].spacing(8.0);

    column![toolbar, space::vertical(), export_tools]
        .padding(8.0)
        .width(Length::Fill)
        .height(100.0)
        .into()
}
