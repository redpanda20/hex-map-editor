use iced::{
    Element, Rectangle, Renderer,
    advanced::{
        Layout,
        widget::{Operation, Tree, operation},
    },
    widget::Id,
};

pub fn subtree_is_focused<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) -> bool {
    struct FindFocused {
        focused: bool,
    }

    impl Operation for FindFocused {
        fn focusable(
            &mut self,
            _id: Option<&Id>,
            _bounds: Rectangle,
            state: &mut dyn operation::Focusable,
        ) {
            self.focused |= state.is_focused();
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
    }

    let mut op = FindFocused { focused: false };
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut op);
    op.focused
}
