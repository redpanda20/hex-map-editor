//! A wrapper that shows a floating menu when its content is right-clicked.

use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::{
    Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector, keyboard, touch,
    window,
};

use crate::app::Message;

/// Displays `menu` at the cursor position when `content` is right clicked.
pub fn context_menu<'a>(
    content: impl Into<Element<'a, Message>>,
    menu: impl Fn() -> Element<'a, Message> + 'a,
) -> Element<'a, Message> {
    ContextMenu::new(content.into(), menu).into()
}

// Lives in the [`Tree`].
#[derive(Debug, Default, Clone, Copy)]
struct State {
    open: bool,
    position: Point,
}

struct ContextMenu<'a> {
    content: Element<'a, Message>,
    menu: Box<dyn Fn() -> Element<'a, Message> + 'a>,
    // Built during `layout`, consumed by `overlay`. Only `Some` while open.
    menu_content: Option<Element<'a, Message>>,
}

impl<'a> ContextMenu<'a> {
    fn new(content: Element<'a, Message>, menu: impl Fn() -> Element<'a, Message> + 'a) -> Self {
        Self {
            content,
            menu: Box::new(menu),
            menu_content: None,
        }
    }
}

impl<'a> Widget<Message, Theme, Renderer> for ContextMenu<'a> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let open = tree.state.downcast_ref::<State>().open;

        if open {
            self.menu_content = Some((self.menu)());
            let children = [&self.content, self.menu_content.as_ref().unwrap()];
            tree.diff_children(&children);
        } else {
            self.menu_content = None;
            tree.diff_children(std::slice::from_ref(&self.content));
        }

        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event {
            if cursor.is_over(layout.bounds()) {
                let state = tree.state.downcast_mut::<State>();
                state.position = cursor.position().unwrap_or_default();
                state.open = !state.open;

                shell.capture_event();
                shell.request_redraw();
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
            layout,
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
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state: &mut State = tree.state.downcast_mut();

        if !state.open {
            return None;
        }

        let content = self.menu_content.as_mut()?;

        Some(
            MenuOverlay {
                position: state.position + translation,
                state,
                tree: &mut tree.children[1],
                content,
            }
            .overlay(),
        )
    }
}

impl<'a> From<ContextMenu<'a>> for Element<'a, Message> {
    fn from(widget: ContextMenu<'a>) -> Self {
        Self::new(widget)
    }
}

/// The floating overlay shown while a [`ContextMenu`] is open.
///
/// 'short is the lifetime of a frame
/// 'long is the lifetime of the content borrowed from
struct MenuOverlay<'short, 'long: 'short> {
    position: Point,
    state: &'short mut State,
    tree: &'short mut Tree,
    content: &'short mut Element<'long, Message>,
}

impl<'short, 'long> MenuOverlay<'short, 'long> {
    fn overlay(self) -> overlay::Element<'short, Message, Theme, Renderer> {
        overlay::Element::new(Box::new(self))
    }
}

impl<'short, 'long> overlay::Overlay<Message, Theme, Renderer> for MenuOverlay<'short, 'long> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);
        let max_size = limits.max();

        let mut content = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);

        // Flip above/left of the click point if menu would overflow the window
        let mut position = self.position;
        if position.x + content.size().width > bounds.width {
            position.x = f32::max(0.0, position.x - content.size().width);
        }
        if position.y + content.size().height > bounds.height {
            position.y = f32::max(0.0, position.y - content.size().height);
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
            .expect("context menu overlay should have a content layout");

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
            .expect("context menu overlay should have a content layout");

        let mut forward = true;
        let mut capture = false;

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                if *key == keyboard::Key::Named(keyboard::key::Named::Escape) =>
            {
                // TODO: Include this as a rebindable option
                self.state.open = false;
                forward = false;
                capture = true;
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left | mouse::Button::Right,
            ))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(content_layout.bounds()) {
                    capture = true;
                } else {
                    // Close menu and let click fall through
                    self.state.open = false;
                    forward = false;
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                // Close after handling input
                self.state.open = false;
                capture = true;
                shell.request_redraw();
            }
            Event::Window(window::Event::Resized { .. }) => {
                self.state.open = false;
                forward = false;
                capture = true;
                shell.request_redraw();
            }
            _ => {}
        }

        if forward {
            self.content.as_widget_mut().update(
                self.tree,
                event,
                content_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
            );
        }
        if capture {
            shell.capture_event();
        }
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        let content_layout = layout
            .children()
            .next()
            .expect("context menu overlay should have a content layout");

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
            .expect("context menu overlay should have a content layout");

        self.content.as_widget().mouse_interaction(
            self.tree,
            content_layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
}
