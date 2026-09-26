use iced::{
    Element, Task,
    widget::{Tooltip, button, column, container, rule, space, text, tooltip},
};
use iced_fonts::lucide;

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
        let brush_tool = tool_button(
            lucide::brush(),
            State::should_focus(tool == Tool::Paint),
            Action::SetTool(Tool::Paint),
            "Brush tool (Ctrl + B)",
        );

        let move_tool = tool_button(
            lucide::hand(),
            State::should_focus(tool == Tool::Pan),
            Action::SetTool(Tool::Pan),
            "Move tool (Ctrl + M)",
        );

        let erase_tool = tool_button(
            lucide::eraser(),
            State::should_focus(tool == Tool::Erase),
            Action::SetTool(Tool::Erase),
            "Erase tool (Ctrl + E)",
        );

        let bucket_tool = tool_button(
            lucide::paint_bucket(),
            State::should_focus(tool == Tool::Fill),
            Action::SetTool(Tool::Fill),
            "Bucket fill tool (Ctrl + B)",
        );

        let undo = tool_button(
            lucide::rotate_ccw(),
            State::should_disable(history.can_undo()),
            Action::Undo,
            "Undo last command (Ctrl + Z)",
        );

        let redo = tool_button(
            lucide::rotate_cw(),
            State::should_disable(history.can_redo()),
            Action::Redo,
            "Redo last command (Ctrl + Y)",
        );

        let save_scene = tool_button(
            lucide::file_down(),
            State::active(),
            Action::Save,
            "Save scene to file (Ctrl + S)",
        );

        let load_scene = tool_button(
            lucide::file_up(),
            State::active(),
            Action::Load,
            "Load scene from file (Ctrl + O)",
        );

        let export_png = tool_button(
            lucide::image_plus(),
            State::active(),
            Action::ExportPng,
            "Export scene as a PNG",
        );

        let export_pdf = tool_button(
            lucide::file_text(),
            State::active(),
            Action::ExportPdf,
            "Export scene as a PDF",
        );

        let open_about = tooltip(
            button(lucide::info())
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
            load_scene,
            save_scene,
            rule::horizontal(1),
            export_png,
            export_pdf,
            open_about
        ]
        .spacing(8.0)
        .padding(8.0);

        container(content).style(container::bordered_box).into()
    }
}

enum State {
    Focused,
    Active,
    Inactive,
}
impl State {
    fn active() -> State {
        State::Active
    }
    fn should_focus(is_focused: bool) -> State {
        if is_focused {
            State::Focused
        } else {
            State::Active
        }
    }

    fn should_disable(is_active: bool) -> State {
        if is_active {
            State::Active
        } else {
            State::Inactive
        }
    }
}

fn tool_button<'a>(
    icon: text::Text<'a>,
    state: State,
    action: Action,
    tooltip_text: &'static str,
) -> Tooltip<'a, Message> {
    tooltip(
        button(icon)
            .on_press(Message::Action(action))
            .style(move |theme, status| match state {
                State::Focused => {
                    let mut style = button::background(theme, status);
                    style.text_color = theme.palette().primary;
                    style.background = Some(theme.extended_palette().background.weak.color.into());
                    style
                }
                State::Active => button::subtle(theme, status),
                State::Inactive => button::subtle(theme, button::Status::Disabled),
            }),
        container(tooltip_text)
            .padding(4.0)
            .style(container::bordered_box),
        tooltip::Position::Right,
    )
}
