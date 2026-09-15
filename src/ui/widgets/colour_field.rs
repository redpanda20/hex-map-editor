use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::widget::{Button, Row, button, column, container, row, space, text, text_input};
use iced::{
    Alignment, Background, Border, Color, Element, Event, Length, Point, Rectangle, Renderer, Size,
    Theme, Vector, keyboard, touch,
};

use crate::ui::widgets::{INPUT_WIDTH, colour_picker};
use crate::{
    app::Message,
    domain::colour::{parse_hex_rgb, to_hex_rgb},
};

const SWATCH_SIZE: f32 = 24.0;
const POPOVER_WIDTH: f32 = 240.0;
const POPOVER_GAP: f32 = 6.0;

/// Creates a [`ColourField`] widget.
///
/// Shows a colour swatch and a validated hex field inline.
/// Pressing the swatch opens a popover with additional controls.
/// Only provides on_commit messages, changes are stored locally.
pub fn colour_field<'a>(
    name: impl Into<String>,
    colour: Color,
    on_commit: impl Fn(Color) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    ColourField::new(name.into(), colour, on_commit).into()
}

/// State local to inline element or popover
#[derive(Debug, Clone)]
struct FieldBuffer {
    raw: String,
    synced_colour: Color,
}

impl FieldBuffer {
    fn new(colour: Color) -> Self {
        Self {
            raw: to_hex_rgb(colour),
            synced_colour: colour,
        }
    }

    fn resync(&mut self, colour: Color) {
        if self.synced_colour != colour {
            self.raw = to_hex_rgb(colour);
            self.synced_colour = colour;
        }
    }
}

/// Local state, lives in the [`Tree`].
#[derive(Debug, Clone)]
struct State {
    // Live preview colour of the widget
    live: Color,
    inline: FieldBuffer,
    popover: FieldBuffer,
    popover_open: bool,
}

impl State {
    fn new(colour: Color) -> Self {
        Self {
            live: colour,
            inline: FieldBuffer::new(colour),
            popover: FieldBuffer::new(colour),
            popover_open: false,
        }
    }

    /// Updates the live colour and keeps both hex buffers in step with
    /// it (unless a buffer is mid-edit - see `FieldBuffer::resync`).
    fn set_live(&mut self, colour: Color) {
        self.live = colour;
        self.inline.resync(colour);
        self.popover.resync(colour);
    }
}

/// Messages internal to a [`ColourField`].
#[derive(Debug, Clone)]
enum Internal {
    ToggleOpen,
    InlineChange(String),
    InlineSubmit,
    PopoverChange(String),
    PopoverSubmit,
    /// A live, uncommitted colour from dragging the SV square/hue
    /// slider. Internal-only: updates the preview, nothing more.
    PickerChange(Color),
    /// The SV square/hue slider drag finished on this colour.
    PickerCommit(Color),
}

/// Applies one [`Internal`] message to `State`,
/// publishing `on_commit` only for a finalised edit.
fn handle_internal<F>(
    message: Internal,
    mode: &mut State,
    on_commit: F,
    shell: &mut Shell<'_, Message>,
) where
    F: Fn(Color) -> Message,
{
    match message {
        Internal::ToggleOpen => {
            mode.popover_open = !mode.popover_open;
            shell.invalidate_layout();
            shell.request_redraw();
        }
        Internal::InlineChange(raw) => {
            mode.inline.raw = raw;
            shell.request_redraw();
        }
        Internal::InlineSubmit => {
            if let Some([r, g, b]) = parse_hex_rgb(&mode.inline.raw) {
                let colour = Color::from_rgba8(r, g, b, mode.live.a);
                mode.set_live(colour);
                shell.publish(on_commit(colour));
                shell.request_redraw();
            }
        }
        Internal::PopoverChange(raw) => {
            mode.popover.raw = raw;
            shell.request_redraw();
        }
        Internal::PopoverSubmit => {
            if let Some([r, g, b]) = parse_hex_rgb(&mode.popover.raw) {
                let colour = Color::from_rgba8(r, g, b, mode.live.a);
                mode.set_live(colour);
                shell.publish(on_commit(colour));
                shell.request_redraw();
            }
        }
        Internal::PickerChange(colour) => {
            mode.set_live(colour);
            shell.request_redraw();
        }
        Internal::PickerCommit(colour) => {
            mode.set_live(colour);
            shell.publish(on_commit(colour));
            shell.request_redraw();
        }
    }
}

fn hex_field<'a>(
    raw: &str,
    on_change: impl Fn(String) -> Internal + 'a,
    on_submit: Internal,
) -> Row<'a, Internal> {
    let is_valid = parse_hex_rgb(raw).is_some();

    row![
        text("#").style(text::secondary),
        text_input("", raw)
            .on_input(on_change)
            .on_submit(on_submit)
            .style(move |theme: &Theme, status| {
                let mut style = text_input::default(theme, status);

                if !is_valid {
                    style.border.color = theme.palette().danger;
                    style.border.width = 1.0;
                }

                style
            })
            .width(INPUT_WIDTH)
    ]
    .spacing(4)
    .align_y(Alignment::Center)
}

fn swatch<'a>(colour: Color) -> Button<'a, Internal> {
    button(
        space()
            .width(Length::Fixed(SWATCH_SIZE))
            .height(Length::Fixed(SWATCH_SIZE)),
    )
    .on_press(Internal::ToggleOpen)
    .padding(0)
    .style(move |theme: &Theme, status| {
        let border_colour = match status {
            button::Status::Hovered | button::Status::Pressed => {
                theme.extended_palette().background.stronger.color
            }
            _ => theme.extended_palette().background.strong.color,
        };

        button::Style {
            background: Some(Background::Color(colour)),
            border: Border {
                color: border_colour,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..button::Style::default()
        }
    })
}

fn inline_content<'a>(name: &str, colour: Color, raw: &str) -> Element<'a, Internal> {
    row![
        text(name.to_string()).style(text::secondary),
        space::horizontal(),
        swatch(colour),
        hex_field(raw, Internal::InlineChange, Internal::InlineSubmit)
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn popover_content<'a>(colour: Color, raw: &str) -> Element<'a, Internal> {
    let picker = colour_picker(colour, Internal::PickerChange, Internal::PickerCommit);
    let field = hex_field(raw, Internal::PopoverChange, Internal::PopoverSubmit);

    container(column![picker, field].spacing(8))
        .padding(8)
        .style(container::rounded_box)
        .width(Length::Fixed(POPOVER_WIDTH))
        .height(Length::Shrink)
        .into()
}

struct ColourField<'a, F>
where
    F: Fn(Color) -> Message + Copy + 'a,
{
    name: String,
    colour: Color,
    on_commit: F,
    content: Element<'a, Internal>,
    // Built during `layout`, consumed by `overlay`. Only `Some` while open.
    popover_content: Option<Element<'a, Internal>>,
}

impl<'a, F> ColourField<'a, F>
where
    F: Fn(Color) -> Message + Copy + 'a,
{
    fn new(name: String, colour: Color, on_commit: F) -> Self {
        let content = inline_content(&name, colour, &to_hex_rgb(colour));

        Self {
            name,
            colour,
            on_commit,
            content,
            popover_content: None,
        }
    }
}

impl<'a, F> Widget<Message, Theme, Renderer> for ColourField<'a, F>
where
    F: Fn(Color) -> Message + Copy + 'a,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.colour))
    }

    fn children(&self) -> Vec<Tree> {
        match &self.popover_content {
            Some(popover) => vec![Tree::new(&self.content), Tree::new(popover)],
            None => vec![Tree::new(&self.content)],
        }
    }

    fn diff(&self, tree: &mut Tree) {
        // Adopt the new external colour, unless the user is mid-edit -
        // in which case this is a no-op (see `FieldBuffer::resync`, and
        // note `live` itself only tracks *our own* edits, so overwriting
        // it here whenever the prop actually changed is always correct).
        let mode = tree.state.downcast_mut::<State>();
        if mode.live != self.colour {
            mode.set_live(self.colour);
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mode = tree.state.downcast_ref::<State>().clone();

        self.content = inline_content(&self.name, mode.live, &mode.inline.raw);

        if mode.popover_open {
            self.popover_content = Some(popover_content(mode.live, &mode.popover.raw));

            let children = [&self.content, self.popover_content.as_ref().unwrap()];
            tree.diff_children(&children);
        } else {
            self.popover_content = None;
            tree.diff_children(std::slice::from_ref(&self.content));
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

        let mode = tree.state.downcast_mut::<State>();
        for message in internal_messages {
            handle_internal(message, mode, self.on_commit, shell);
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let mode: &mut State = tree.state.downcast_mut();

        if !mode.popover_open {
            return None;
        }

        let content = self.popover_content.as_mut()?;
        let bounds = layout.bounds();

        Some(
            ColourPopoverOverlay {
                anchor: bounds.position() + translation,
                anchor_height: bounds.height,
                mode,
                on_commit: self.on_commit,
                tree: &mut tree.children[1],
                content,
            }
            .overlay(),
        )
    }
}

impl<'a, F> From<ColourField<'a, F>> for Element<'a, Message>
where
    F: Fn(Color) -> Message + Copy + 'a,
{
    fn from(field: ColourField<'a, F>) -> Self {
        Self::new(field)
    }
}

/// The floating overlay shown while a [`ColourField`]'s popover is open.
///
/// 'short is the lifetime of a frame
/// 'long is the lifetime of the content borrowed from
struct ColourPopoverOverlay<'short, 'long, F>
where
    'long: 'short,
    F: Fn(Color) -> Message + Copy,
{
    anchor: Point,
    anchor_height: f32,
    mode: &'short mut State,
    on_commit: F,
    tree: &'short mut Tree,
    content: &'short mut Element<'long, Internal>,
}

impl<'short, 'long, F> ColourPopoverOverlay<'short, 'long, F>
where
    F: Fn(Color) -> Message + Copy + 'short,
{
    fn overlay(self) -> overlay::Element<'short, Message, Theme, Renderer> {
        overlay::Element::new(Box::new(self))
    }
}

impl<'short, 'long, F> overlay::Overlay<Message, Theme, Renderer>
    for ColourPopoverOverlay<'short, 'long, F>
where
    F: Fn(Color) -> Message + Copy,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);
        let max_size = limits.max();

        let mut content = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);

        // Anchor below & right pf the spawning field.
        // Flip left, then up if there is no room
        let mut position = Point {
            x: self.anchor.x,
            y: self.anchor.y + self.anchor_height + POPOVER_GAP,
        };

        if position.x + content.size().width > bounds.width {
            position.x = f32::max(0.0, bounds.width - content.size().width);
        }
        if position.y + content.size().height > bounds.height {
            position.y = f32::max(0.0, self.anchor.y - POPOVER_GAP - content.size().height);
        }

        content.move_to_mut(position);

        layout::Node::with_children(max_size, vec![content])
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let content_layout = layout
            .children()
            .next()
            .expect("colour popover overlay should have a content layout");

        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            content_layout,
            cursor,
            &layout.bounds(),
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let content_layout = layout
            .children()
            .next()
            .expect("colour popover overlay should have a content layout");

        let mut forward = true;

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                if *key == keyboard::Key::Named(keyboard::key::Named::Escape) =>
            {
                self.mode.popover_open = false;
                forward = false;
                shell.invalidate_layout();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left | mouse::Button::Right,
            ))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if !cursor.is_over(content_layout.bounds()) =>
            {
                // Close and let the click fall through
                self.mode.popover_open = false;
                forward = false;
                shell.invalidate_layout();
                shell.request_redraw();
            }
            _ => {}
        }

        if forward {
            let mut internal_messages = Vec::new();
            let mut local_shell = Shell::new(&mut internal_messages);

            self.content.as_widget_mut().update(
                self.tree,
                event,
                content_layout,
                cursor,
                renderer,
                clipboard,
                &mut local_shell,
                &layout.bounds(),
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
                handle_internal(message, self.mode, self.on_commit, shell);
            }
        }
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        let content_layout = layout
            .children()
            .next()
            .expect("colour popover overlay should have a content layout");

        self.content
            .as_widget_mut()
            .operate(self.tree, content_layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let content_layout = layout
            .children()
            .next()
            .expect("colour popover overlay should have a content layout");

        self.content.as_widget().mouse_interaction(
            self.tree,
            content_layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
}
