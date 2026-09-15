use std::ops::RangeInclusive;

use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, renderer};
use iced::widget::{column, row, slider, space, text};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme};

use crate::app::Message;

/// Creates a [`BoundedFloatField`] widget.
pub fn bounded_float_field<'a>(
    name: impl Into<String>,
    starting_value: f64,
    range: RangeInclusive<f64>,
    on_submit: impl Fn(f64) -> Message + 'a,
) -> Element<'a, Message> {
    BoundedFloatField::new(name.into(), starting_value, range, on_submit).into()
}

/// Lives in the [`Tree`].
#[derive(Debug, Default, Clone)]
struct State {
    value: f64,
}

/// Messages internal to a [`BoundedFloatField`].
#[derive(Debug, Clone)]
enum Internal {
    Change(f64),
    Submit,
}

struct BoundedFloatField<'a> {
    name: String,
    starting_value: f64,
    range: RangeInclusive<f64>,
    on_submit: Box<dyn Fn(f64) -> Message + 'a>,
    content: Element<'a, Internal>,
}

impl<'a> BoundedFloatField<'a> {
    fn new(
        name: String,
        starting_value: f64,
        range: RangeInclusive<f64>,
        on_submit: impl Fn(f64) -> Message + 'a,
    ) -> Self {
        let content = content(name.clone(), starting_value, range.clone());
        Self {
            name,
            starting_value,
            range,
            on_submit: Box::new(on_submit),
            content,
        }
    }

    fn rebuild_content(&mut self, value: f64) {
        self.content = content(self.name.clone(), value, self.range.clone())
    }
}

fn content<'a>(name: String, value: f64, range: RangeInclusive<f64>) -> Element<'a, Internal> {
    let step = (*range.end() - *range.start()) / 100.0;

    let slider = slider(range, value, Internal::Change)
        .on_release(Internal::Submit)
        .step(step);

    column![
        row![
            text(name).style(text::secondary),
            space::horizontal(),
            text!("{value:.2}")
        ],
        slider
    ]
    .spacing(8)
    .into()
}

impl<'a> Widget<Message, Theme, Renderer> for BoundedFloatField<'a> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            value: self.starting_value,
        })
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

        // Forward shell from child widgets
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
                Internal::Change(value) => {
                    let state = tree.state.downcast_mut::<State>();

                    state.value = value;
                    self.rebuild_content(value);

                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                Internal::Submit => {
                    let state = tree.state.downcast_mut::<State>();

                    shell.publish((self.on_submit)(state.value));
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

impl<'a> From<BoundedFloatField<'a>> for Element<'a, Message> {
    fn from(field: BoundedFloatField<'a>) -> Self {
        Self::new(field)
    }
}
