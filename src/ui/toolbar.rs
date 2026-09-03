use iced::{
    Element, Task,
    widget::{button, column, container, rule, space, tooltip},
};
use iced_fonts::bootstrap;

use crate::{
    app::{Action, Message},
    domain::{History, Tool},
};

#[derive(Debug, Clone)]
pub enum ToolbarMessage {}

#[derive(Debug, Default, Clone)]
pub struct Toolbar {}

impl Toolbar {
    pub fn update(&mut self, _message: ToolbarMessage) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(&self, tool: Tool, history: &'a History) -> Element<'a, Message> {
        let tool_button = |icon, selected, message, tooltip_text| {
            tooltip(
                button(icon)
                    .on_press(Message::Action(message))
                    .style(move |theme, status| {
                        button::background(
                            theme,
                            if selected {
                                button::Status::Disabled
                            } else {
                                status
                            },
                        )
                    }),
                container(tooltip_text)
                    .padding(4.0)
                    .style(container::bordered_box),
                tooltip::Position::Right,
            )
        };

        let brush_tool = tool_button(
            bootstrap::brush(),
            tool == Tool::Paint,
            Action::SetTool(Tool::Paint),
            "Brush tool (Ctrl + B)",
        );

        let move_tool = tool_button(
            bootstrap::arrows_move(),
            tool == Tool::Pan,
            Action::SetTool(Tool::Pan),
            "Move tool (Ctrl + M)",
        );

        let erase_tool = tool_button(
            bootstrap::eraser_fill(),
            tool == Tool::Erase,
            Action::SetTool(Tool::Erase),
            "Erase tool (Ctrl + E)",
        );

        let bucket_tool = tool_button(
            bootstrap::paint_bucket(),
            tool == Tool::Fill,
            Action::SetTool(Tool::Fill),
            "Bucket fill tool (Ctrl + B)",
        );

        let undo = tool_button(
            bootstrap::arrow_counterclockwise(),
            !history.can_undo(),
            Action::Undo,
            "Undo last command (Ctrl + Z)",
        );

        let redo = tool_button(
            bootstrap::arrow_clockwise(),
            !history.can_redo(),
            Action::Redo,
            "Redo last command (Ctrl + Y)",
        );

        let save_scene = tool_button(
            bootstrap::floppy_fill(),
            false,
            Action::Save,
            "Save scene to file (Ctrl + S)",
        );

        let load_scene = tool_button(
            bootstrap::folder_fill(),
            false,
            Action::Load,
            "Load scene from file (Ctrl + O)",
        );

        let export_png = tool_button(
            bootstrap::file_earmark_image(),
            false,
            Action::ExportPng,
            "Export scene as a PNG",
        );

        let open_about = tooltip(
            button(bootstrap::file_richtext_fill())
                .on_press(Message::About(crate::ui::AboutMessage::Show))
                .style(button::text),
            container("About this application.")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Right,
        );

        let content = column![
            rule::horizontal(1),
            brush_tool,
            move_tool,
            erase_tool,
            bucket_tool,
            rule::horizontal(1),
            undo,
            redo,
            space::vertical(),
            save_scene,
            load_scene,
            export_png,
            open_about
        ]
        .spacing(8.0)
        .padding(8.0);

        container(content).style(container::bordered_box).into()
    }
}
