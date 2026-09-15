use std::ops::RangeInclusive;

use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, renderer};
use iced::widget::{row, text, text_input};
use iced::{Alignment, Element, Event, Length, Rectangle, Renderer, Size, Theme};

use crate::app::Message;

/// Creates a [`BoundedIntegerField`] widget.
pub fn bounded_integer_field<'a>(
    name: impl Into<String>,
    starting_value: u64,
    range: RangeInclusive<u64>,
    on_submit: impl Fn(u64) -> Message + 'a,
) -> Element<'a, Message> {
    BoundedIntegerField::new(name.into(), starting_value, range, on_submit).into()
}

/// Local state, Lives in the [`Tree`].
#[derive(Debug, Clone)]
struct Mode {
    raw: String,
}

/// Messages internal to a [`BoundedFloatField`].
#[derive(Debug, Clone)]
enum Internal {
    Change(String),
    Submit,
}

struct BoundedIntegerField<'a> {
    name: String,
    value: u64,
    range: RangeInclusive<u64>,
    on_submit: Box<dyn Fn(u64) -> Message + 'a>,
    content: Element<'a, Internal>,
}

impl<'a> BoundedIntegerField<'a> {
    fn new(
        name: String,
        value: u64,
        range: RangeInclusive<u64>,
        on_submit: impl Fn(u64) -> Message + 'a,
    ) -> Self {
        let content = editing_content(name.clone(), &value.to_string(), range.clone());
        Self {
            name,
            value,
            range,
            on_submit: Box::new(on_submit),
            content,
        }
    }
}

fn parse_input(text: &str) -> Option<u64> {
    text.trim().parse::<u64>().ok()
}

fn editing_content<'a>(
    name: String,
    text_value: &str,
    range: RangeInclusive<u64>,
) -> Element<'a, Internal> {
    let is_valid = parse_input(text_value)
        .map(|num| range.contains(&num))
        .unwrap_or(false);

    row![
        text(name)
            .style(text::secondary)
            .width(Length::Fixed(100.0)),
        text_input("", text_value)
            .on_input(Internal::Change)
            .on_submit(Internal::Submit)
            .style(move |theme: &Theme, status| {
                let mut style = text_input::default(theme, status);

                if !is_valid {
                    style.border.color = theme.palette().danger;
                    style.border.width = 1.0;
                }

                style
            })
            .width(Length::Fill)
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .into()
}

impl<'a> Widget<Message, Theme, Renderer> for BoundedIntegerField<'a> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Mode>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Mode {
            raw: self.value.to_string(),
        })
    }

    fn diff(&self, _tree: &mut Tree) {}

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mode = tree.state.downcast_ref::<Mode>().clone();

        self.content = editing_content(self.name.clone(), &mode.raw, self.range.clone());
        tree.diff_children(std::slice::from_ref(&self.content));

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
                Internal::Change(new_text) => {
                    let Mode { raw, .. } = tree.state.downcast_mut::<Mode>();
                    *raw = new_text;

                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                Internal::Submit => {
                    let Mode { raw, .. } = tree.state.downcast_ref::<Mode>();

                    let submitted = parse_input(raw);

                    if let Some(value) = submitted
                        && self.range.contains(&value)
                    {
                        shell.publish((self.on_submit)(value));

                        shell.invalidate_layout();
                        shell.request_redraw();
                    }
                }
            }
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
}

impl<'a> From<BoundedIntegerField<'a>> for Element<'a, Message> {
    fn from(field: BoundedIntegerField<'a>) -> Self {
        Self::new(field)
    }
}
