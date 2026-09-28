use iced::{
    Element, Length, Task,
    widget::{button, column, row},
};
use iced_fonts::lucide;

use crate::{
    app::{Action, Message},
    domain::{History, Tool},
    theme,
};

#[derive(Debug, Clone)]
pub enum ToolbarMessage {}

#[derive(Debug, Default, Clone)]
pub struct Toolbar {}

impl Toolbar {
    pub fn update(&mut self, _message: ToolbarMessage) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(&self, tool: Tool, _: &'a History) -> Element<'a, Message> {
        let tools = [
            (lucide::hand(), Tool::Pan),
            (lucide::brush(), Tool::Paint),
            (lucide::eraser(), Tool::Erase),
            (lucide::paint_bucket(), Tool::Fill),
        ];

        let content = column(tools.into_iter().map(|t| {
            row![
                theme::tool_marker(tool == t.1),
                button(t.0)
                    .on_press_maybe((tool != t.1).then_some(Message::Action(Action::SetTool(t.1))))
                    .style(theme::tool_button)
            ]
            .height(Length::Shrink)
            .into()
        }))
        .spacing(8);

        content.into()
    }
}
