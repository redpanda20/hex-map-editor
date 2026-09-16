use std::ops::RangeInclusive;

use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, renderer};
use iced::widget::{column, row, space, text, text_input};
use iced::{Alignment, Element, Event, Length, Rectangle, Renderer, Size, Theme};

use crate::ui::widgets::INPUT_WIDTH;
use crate::ui::widgets::helper::subtree_is_focused;

/// Creates a [`BoundedIntegerField`] widget.
pub fn bounded_integer_field<'a, Message>(
    name: impl Into<String>,
    starting_value: u64,
    range: RangeInclusive<u64>,
    on_submit: impl Fn(u64) -> Message + 'a,
) -> BoundedIntegerField<'a, Message>
where
    Message: 'a,
{
    BoundedIntegerField::new(name.into(), starting_value, range, on_submit)
}

#[derive(Debug, Default, Clone, Copy)]
enum FieldLayout {
    #[default]
    Horizontal,
    Vertical,
}

/// Local state, Lives in the [`Tree`].
#[derive(Debug, Clone)]
struct State {
    raw: String,
    committed: u64,
}

/// Messages internal to a [`BoundedFloatField`].
#[derive(Debug, Clone)]
enum Internal {
    Change(String),
    Submit,
}

pub struct BoundedIntegerField<'a, Message> {
    name: String,
    value: u64,
    range: RangeInclusive<u64>,
    on_submit: Box<dyn Fn(u64) -> Message + 'a>,
    layout: FieldLayout,
    content: Element<'a, Internal>,
}

impl<'a, Message> BoundedIntegerField<'a, Message> {
    pub fn new(
        name: String,
        value: u64,
        range: RangeInclusive<u64>,
        on_submit: impl Fn(u64) -> Message + 'a,
    ) -> Self {
        let layout = FieldLayout::default();
        let content = content(name.clone(), &value.to_string(), range.clone(), layout);
        Self {
            name,
            value,
            range,
            on_submit: Box::new(on_submit),
            layout,
            content,
        }
    }

    pub fn horizontal(mut self) -> Self {
        self.layout = FieldLayout::Horizontal;
        self.content = content(
            self.name.clone(),
            &self.value.to_string(),
            self.range.clone(),
            self.layout,
        );
        self
    }

    pub fn vertical(mut self) -> Self {
        self.layout = FieldLayout::Vertical;
        self.content = content(
            self.name.clone(),
            &self.value.to_string(),
            self.range.clone(),
            self.layout,
        );
        self
    }

    fn rebuild_content(&mut self, text_value: &str) {
        self.content = content(
            self.name.clone(),
            text_value,
            self.range.clone(),
            self.layout,
        )
    }
}

fn parse_input(text: &str) -> Option<u64> {
    text.trim().parse::<u64>().ok()
}

fn content<'a>(
    name: String,
    text_value: &str,
    range: RangeInclusive<u64>,
    layout: FieldLayout,
) -> Element<'a, Internal> {
    let is_valid = parse_input(text_value)
        .map(|num| range.contains(&num))
        .unwrap_or(false);

    let title = text(name).style(text::secondary);

    let input = text_input("", text_value)
        .on_input(Internal::Change)
        .on_submit(Internal::Submit)
        .style(move |theme: &Theme, status| {
            let mut style = text_input::default(theme, status);

            if !is_valid {
                style.border.color = theme.palette().danger;
                style.border.width = 1.0;
            }

            style
        });

    match layout {
        FieldLayout::Horizontal => row![title, space::horizontal(), input.width(INPUT_WIDTH)]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into(),
        FieldLayout::Vertical => column![title, input].spacing(4).width(Length::Fill).into(),
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for BoundedIntegerField<'a, Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            raw: self.value.to_string(),
            committed: self.value,
        })
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        if state.committed != self.value {
            state.committed = self.value;
            state.raw = self.value.to_string();
        }

        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let State { raw, .. } = tree.state.downcast_ref::<State>();

        self.rebuild_content(raw);

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

        let was_focused = subtree_is_focused(
            &mut self.content,
            &mut tree.children[0],
            content_layout,
            renderer,
        );

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

        let is_focused = subtree_is_focused(
            &mut self.content,
            &mut tree.children[0],
            content_layout,
            renderer,
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
                    let State { raw, .. } = tree.state.downcast_mut::<State>();

                    *raw = new_text;

                    // Invalidating layout rebuilds content
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                Internal::Submit => {
                    let State { raw, .. } = tree.state.downcast_ref::<State>();

                    let submitted = parse_input(raw);

                    if let Some(value) = submitted
                        && self.range.contains(&value)
                    {
                        shell.publish((self.on_submit)(value));
                    }
                }
            }
        }

        if was_focused && !is_focused {
            let State { raw, committed } = tree.state.downcast_mut::<State>();
            *raw = committed.to_string();

            // Invalidating layout rebuilds content
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
}

impl<'a, Message> From<BoundedIntegerField<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(field: BoundedIntegerField<'a, Message>) -> Self {
        Self::new(field)
    }
}
