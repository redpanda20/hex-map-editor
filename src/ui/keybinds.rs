use std::{collections::HashMap, hash::Hash};

use iced::{
    Subscription,
    keyboard::{self, Modifiers},
};

use crate::{
    app::{Action, Message},
    domain::Tool,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Binding {
    key: keyboard::Key,
    modifiers: keyboard::Modifiers,
}

impl Binding {
    pub fn new<'a>(key: impl Into<&'a str>, modifiers: Modifiers) -> Self {
        Self {
            key: keyboard::Key::Character(key.into().into()),
            modifiers,
        }
    }

    pub fn ctrl<'a>(key: impl Into<&'a str>) -> Self {
        Self::new(key, Modifiers::CTRL)
    }

    pub fn ctrl_shift<'a>(key: impl Into<&'a str>) -> Self {
        Self::new(key, Modifiers::CTRL | Modifiers::SHIFT)
    }

    fn from_event(event: keyboard::Event) -> Option<Self> {
        match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => Some(Binding { key, modifiers }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum KeybindMessage {
    Bind { binding: Binding, action: Action },
    Unbind { binding: Binding },
}

#[derive(Debug, Clone)]
pub struct Keybinds {
    bindings: HashMap<Binding, Action>,
    /// Changed whenever the bindings change.
    /// Used by iced subscriptions for cache invalidation
    revision: u64,
}

#[derive(Debug, Clone)]
struct KeybindsWrapper(Keybinds);

impl Hash for KeybindsWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.revision.hash(state);
    }
}

impl Keybinds {
    /// Create a new keybind layout with existing bindings
    pub fn new_with(bindings: HashMap<Binding, Action>) -> Self {
        Self {
            bindings,
            revision: 0,
        }
    }

    /// Get the action corresponding to a binding, if it exists
    pub fn action_for(&self, binding: Binding) -> Option<Action> {
        self.bindings.get(&binding).copied()
    }

    /// Set a binding safely
    pub fn set_binding(&mut self, binding: Binding, action: Action) {
        self.bindings.insert(binding, action);
        self.revision = self.revision.wrapping_add(1);
    }

    /// Remove a binding safely
    pub fn unbind(&mut self, binding: &Binding) {
        self.bindings.remove(binding);
        self.revision = self.revision.wrapping_add(1);
    }
}

impl Keybinds {
    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen()
            .filter_map(Binding::from_event)
            .with(KeybindsWrapper(self.clone()))
            .filter_map(|(keybinds, binding)| keybinds.0.action_for(binding))
            .map(Message::Action)
    }

    pub fn update(&mut self, message: KeybindMessage) {
        match message {
            KeybindMessage::Bind { binding, action } => self.set_binding(binding, action),
            KeybindMessage::Unbind { binding } => self.unbind(&binding),
        };
    }
}

// Used indirectly in Toolbar tooltips.
// If default bindings are changed, they need to be changed over there
impl Default for Keybinds {
    fn default() -> Self {
        let bindings = HashMap::from([
            (Binding::ctrl("z"), Action::Undo),
            (Binding::ctrl_shift("z"), Action::Redo),
            (Binding::ctrl("y"), Action::Redo),
            (Binding::ctrl("b"), Action::SetTool(Tool::Paint)),
            (Binding::ctrl("e"), Action::SetTool(Tool::Erase)),
            (Binding::ctrl("m"), Action::SetTool(Tool::Pan)),
            (Binding::ctrl("s"), Action::Save),
            (Binding::ctrl("o"), Action::Load),
        ]);
        Self::new_with(bindings)
    }
}
