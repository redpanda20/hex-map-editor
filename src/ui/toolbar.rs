use iced::{
    Element, Task,
    widget::{button, column, container, rule, space, tooltip},
};
use iced_fonts::bootstrap;

use crate::{
    app::Message,
    domain::{History, HistoryCommand, Scene, SceneMessage, Tool},
    infrastructure::IoProcess,
};

#[derive(Debug, Clone)]
pub enum ToolbarMessage {}

#[derive(Debug, Default, Clone)]
pub struct Toolbar {}

impl Toolbar {
    pub fn update(&mut self, _message: ToolbarMessage) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(&self, scene: &'a Scene, history: &'a History) -> Element<'a, Message> {
        let brush_tool = button(bootstrap::brush())
            .on_press(Message::Scene(SceneMessage::ChangeTool(Tool::Paint)))
            .style(move |theme, mut status| {
                if scene.tool == Tool::Paint {
                    status = button::Status::Disabled
                };
                button::background(theme, status)
            });
        let brush_tool = tooltip(
            brush_tool,
            container("Brush tool")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Right,
        );

        let move_tool = button(bootstrap::arrows_move())
            .on_press(Message::Scene(SceneMessage::ChangeTool(Tool::Pan)))
            .style(move |theme, mut status| {
                if scene.tool == Tool::Pan {
                    status = button::Status::Disabled
                };
                button::background(theme, status)
            });
        let move_tool = tooltip(
            move_tool,
            container("Move tool")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Right,
        );

        let erase_tool = button(bootstrap::eraser_fill())
            .on_press(Message::Scene(SceneMessage::ChangeTool(Tool::Erase)))
            .style(move |theme, mut status| {
                if scene.tool == Tool::Erase {
                    status = button::Status::Disabled
                };
                button::background(theme, status)
            });
        let erase_tool = tooltip(
            erase_tool,
            container("Erase tool")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Right,
        );

        let paint_tool = button(bootstrap::paint_bucket())
            .on_press(Message::Scene(SceneMessage::ChangeTool(Tool::Fill)))
            .style(move |theme, mut status| {
                if scene.tool == Tool::Fill {
                    status = button::Status::Disabled
                };
                button::background(theme, status)
            });
        let paint_tool = tooltip(
            paint_tool,
            container("Bucket fill tool")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Right,
        );

        let undo = button(bootstrap::arrow_counterclockwise())
            .on_press(Message::History(HistoryCommand::Undo))
            .style(move |theme, mut status| {
                if !history.can_undo() {
                    status = button::Status::Disabled
                }
                button::subtle(theme, status)
            });
        let undo = tooltip(
            undo,
            container("Undo last command")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Right,
        );
        let redo = button(bootstrap::arrow_clockwise())
            .on_press(Message::History(HistoryCommand::Redo))
            .style(move |theme, mut status| {
                if !history.can_redo() {
                    status = button::Status::Disabled
                }
                button::subtle(theme, status)
            });
        let redo = tooltip(
            redo,
            container("Redo last command")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Right,
        );

        let save_scene = button(bootstrap::floppy_fill())
            .style(button::subtle)
            .on_press(Message::Save(IoProcess::Start));
        let save_scene = tooltip(
            save_scene,
            container("Save scene to file")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Right,
        );
        let load_scene = button(bootstrap::folder_fill())
            .style(button::subtle)
            .on_press(Message::Load(IoProcess::Start));
        let load_scene = tooltip(
            load_scene,
            container("Load scene from file")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Right,
        );

        let export_png =
            button(bootstrap::file_earmark_image()).on_press(Message::Export(IoProcess::Start));
        let export_png = tooltip(
            export_png,
            container("Export scene as a PNG")
                .padding(4.0)
                .style(container::bordered_box),
            tooltip::Position::Right,
        );

        let content = column![
            rule::horizontal(1),
            brush_tool,
            move_tool,
            erase_tool,
            paint_tool,
            rule::horizontal(1),
            undo,
            redo,
            space::vertical(),
            save_scene,
            load_scene,
            export_png
        ]
        .spacing(8.0)
        .padding(8.0);

        container(content).style(container::bordered_box).into()
    }
}
