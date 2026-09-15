use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, renderer};
use iced::widget::{button, column, container, space, text, text_input};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme};

use crate::app::Message;

/// Creates a [`TextField`] widget.
///
/// Displays `starting_value` as an inline label. When interacted
/// it becomes a real [`text_input`], and any edits are persisted
/// until the user commits them (by pressing enter), or are dropped.
pub fn inline_text_field<'a>(
    starting_value: &'a str,
    on_submit: impl Fn(&str) -> Message + 'a,
) -> Element<'a, Message> {
    TextField::new(starting_value, on_submit).into()
}

type Paragraph = <Renderer as TextRenderer>::Paragraph;

/// The widget-local, persistent state of a [`TextField`].
/// Lives in the [`Tree`].
#[derive(Debug, Clone)]
enum Mode {
    Idle,
    Editing {
        value: String,
        // Set for exactly one `layout` pass after entering this mode,
        // so the freshly-mounted `text_input` can be focused.
        focus_pending: bool,
    },
}

/// Messages internal to a [`TextField`].
#[derive(Debug, Clone)]
enum Internal {
    Edit,
    Change(String),
    Submit,
}

struct TextField<'a> {
    value: &'a str,
    on_submit: Box<dyn Fn(&str) -> Message + 'a>,
    content: Element<'a, Internal>,
}

impl<'a> TextField<'a> {
    fn new(value: &'a str, on_submit: impl Fn(&str) -> Message + 'a) -> Self {
        Self {
            value,
            on_submit: Box::new(on_submit),
            content: idle_content(value),
        }
    }
}

fn idle_content<'a>(value: &'a str) -> Element<'a, Internal> {
    let text_content = column![
        // TODO: Unbake this value
        // 19.0 happens to be the height that matches text_input
        text(value).height(19.0),
        container(space())
            .width(Length::Fill)
            .height(2)
            .style(|theme: &Theme| {
                container::Style {
                    background: Some(theme.palette().primary.into()),
                    ..Default::default()
                }
            }),
    ]
    .width(Length::Shrink);

    button(text_content)
        .on_press(Internal::Edit)
        .style(button::text)
        .into()
}

fn editing_content<'a>(value: &str) -> Element<'a, Internal> {
    text_input("", value)
        .on_input(Internal::Change)
        .on_submit(Internal::Submit)
        .into()
}

fn text_input_state(tree: &mut Tree) -> &mut text_input::State<Paragraph> {
    tree.state.downcast_mut()
}

impl<'a> Widget<Message, Theme, Renderer> for TextField<'a> {
    fn size(&self) -> Size<Length> {
        // Reports a constant size. Otherwise causes invalid
        // sizing: `self.content.as_widget().size_hint()`
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Mode>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Mode::Idle)
    }

    fn diff(&self, _tree: &mut Tree) {
        // Deliberately left as a no-operation.
        //
        // Content depends on `Mode`, which is innaccessible here.
        // We deliberately avoid clearing `tree.children` here,
        // or we would lose the inner `text_input`'s focus every
        // time an unrelated part of the UI causes a fresh `view`.
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mode = tree.state.downcast_ref::<Mode>().clone();

        self.content = match &mode {
            Mode::Idle => idle_content(self.value),
            Mode::Editing { value, .. } => editing_content(value),
        };

        tree.diff_children(std::slice::from_ref(&self.content));

        if matches!(
            mode,
            Mode::Editing {
                focus_pending: true,
                ..
            }
        ) {
            text_input_state(&mut tree.children[0]).focus();
        }

        if let Mode::Editing { focus_pending, .. } = tree.state.downcast_mut::<Mode>() {
            *focus_pending = false;
        }

        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        layout::Node::with_children(node.size(), vec![node])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let content_layout = layout.children().next().unwrap();

        let was_editing = matches!(tree.state.downcast_ref::<Mode>(), Mode::Editing { .. });

        let mut internal_messages = Vec::new();
        let mut local_shell = Shell::new(&mut internal_messages);

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            content_layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );

        shell.request_input_method(local_shell.input_method());
        shell.request_redraw_at(local_shell.redraw_request());
        if local_shell.is_layout_invalid() {
            shell.invalidate_layout();
        }
        if local_shell.are_widgets_invalid() {
            shell.invalidate_widgets();
        }
        if local_shell.is_event_captured() {
            shell.capture_event();
        }

        for message in internal_messages {
            match message {
                Internal::Edit => {
                    *tree.state.downcast_mut::<Mode>() = Mode::Editing {
                        value: self.value.to_string(),
                        focus_pending: true,
                    };

                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                Internal::Change(new_value) => {
                    if let Mode::Editing { value, .. } = tree.state.downcast_mut::<Mode>() {
                        *value = new_value;
                    }

                    shell.request_redraw();
                }
                Internal::Submit => {
                    if let Mode::Editing { value, .. } = tree.state.downcast_ref::<Mode>() {
                        shell.publish((self.on_submit)(value));
                    }

                    *tree.state.downcast_mut::<Mode>() = Mode::Idle;

                    shell.invalidate_layout();
                    shell.request_redraw();
                }
            }
        }

        // Discard uncommitted edits as soon as the text input loses focus.
        let still_editing_unfocused = was_editing
            && matches!(tree.state.downcast_ref::<Mode>(), Mode::Editing { .. })
            && !text_input_state(&mut tree.children[0]).is_focused();

        if still_editing_unfocused {
            *tree.state.downcast_mut::<Mode>() = Mode::Idle;

            shell.invalidate_layout();
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    // No `overlay` override.
    //
    // Content is always either the idle `button` or the `text_input`,
    // neither of which ever produces an overlay.
    //
    // If that changes, forwarding it would need to translate
    // the overlay's `Internal` messages back into `Message`.
}

impl<'a> From<TextField<'a>> for Element<'a, Message> {
    fn from(field: TextField<'a>) -> Self {
        Self::new(field)
    }
}
