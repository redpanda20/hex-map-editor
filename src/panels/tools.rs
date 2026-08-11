use iced::{
    Element,
    widget::{button, column, container, rule, space, tooltip},
};
use iced_fonts::bootstrap;

use crate::{app::Message, state::Tool};

pub fn toolbar_panel(current_tool: &Tool) -> Element<'_, Message> {
    let brush_tool = button(bootstrap::brush())
        .on_press(Message::ChangeTool(Tool::Paint))
        .style(|theme, mut status| {
            if *current_tool == Tool::Paint {
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
        .on_press(Message::ChangeTool(Tool::Pan))
        .style(|theme, mut status| {
            if *current_tool == Tool::Pan {
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
        .on_press(Message::ChangeTool(Tool::Erase))
        .style(|theme, mut status| {
            if *current_tool == Tool::Erase {
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
        .on_press(Message::ChangeTool(Tool::Fill))
        .style(|theme, mut status| {
            if *current_tool == Tool::Fill {
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

    let save_scene = button(bootstrap::floppy_fill())
        .style(button::subtle)
        .on_press(Message::SaveProject);
    let save_scene = tooltip(
        save_scene,
        container("Save scene to file")
            .padding(4.0)
            .style(container::bordered_box),
        tooltip::Position::Right,
    );
    let load_scene = button(bootstrap::folder_fill())
        .style(button::subtle)
        .on_press(Message::LoadProject);
    let load_scene = tooltip(
        load_scene,
        container("Load scene from file")
            .padding(4.0)
            .style(container::bordered_box),
        tooltip::Position::Right,
    );

    let export_png = button(bootstrap::file_earmark_image()).on_press(Message::ExportPng);
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
        space::vertical(),
        save_scene,
        load_scene,
        export_png
    ]
    .spacing(8.0)
    .padding(8.0);

    container(content).style(container::bordered_box).into()
}
