use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, renderer};
use iced::widget::{column, row, space, text, text_input};
use iced::{Alignment, Element, Event, Length, Rectangle, Renderer, Size, Theme};

use crate::ui::widgets::INPUT_WIDTH;
use crate::ui::widgets::helper::subtree_is_focused;

/// Creates a [`UnboundedFloatField`] widget.
pub fn float_field<'a, Message>(
    name: impl Into<String>,
    starting_value: f64,
    on_submit: impl Fn(f64) -> Message + 'a,
) -> FloatField<'a, Message>
where
    Message: 'a,
{
    FloatField::new(name.into(), starting_value, on_submit)
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
    value: f64,
}

/// Messages internal to a [`UnboundedFloatField`].
#[derive(Debug, Clone)]
enum Internal {
    Change(String),
    Submit,
}

pub struct FloatField<'a, Message> {
    name: String,
    value: f64,
    on_submit: Box<dyn Fn(f64) -> Message + 'a>,
    layout: FieldLayout,
    content: Element<'a, Internal>,
}

impl<'a, Message> FloatField<'a, Message> {
    pub fn new(name: String, value: f64, on_submit: impl Fn(f64) -> Message + 'a) -> Self {
        let layout = FieldLayout::default();
        let content = content(name.clone(), &value.to_string(), layout);
        Self {
            name,
            value,
            on_submit: Box::new(on_submit),
            layout,
            content,
        }
    }

    pub fn horizontal(mut self) -> Self {
        self.layout = FieldLayout::Horizontal;
        self.content = content(self.name.clone(), &self.value.to_string(), self.layout);
        self
    }

    pub fn vertical(mut self) -> Self {
        self.layout = FieldLayout::Vertical;
        self.content = content(self.name.clone(), &self.value.to_string(), self.layout);
        self
    }

    fn rebuild_content(&mut self, text_value: &str) {
        self.content = content(self.name.clone(), text_value, self.layout);
    }
}

fn parse_input(text: &str) -> Option<f64> {
    text.trim().parse::<f64>().ok()
}

fn content<'a>(name: String, text_value: &str, layout: FieldLayout) -> Element<'a, Internal> {
    let is_valid = parse_input(text_value).is_some();

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

impl<'a, Message> Widget<Message, Theme, Renderer> for FloatField<'a, Message> {
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
            value: self.value,
        })
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        if state.value != self.value {
            state.value = self.value;
            state.raw = self.value.to_string();
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
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

                    self.rebuild_content(raw);

                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                Internal::Submit => {
                    let State { raw, .. } = tree.state.downcast_ref::<State>();

                    let submitted = parse_input(raw);

                    if let Some(value) = submitted {
                        shell.publish((self.on_submit)(value));
                    }
                }
            }
        }

        if was_focused && !is_focused {
            let State { raw, value } = tree.state.downcast_mut::<State>();
            *raw = value.to_string();

            self.rebuild_content(raw);

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

impl<'a, Message> From<FloatField<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(field: FloatField<'a, Message>) -> Self {
        Self::new(field)
    }
}
