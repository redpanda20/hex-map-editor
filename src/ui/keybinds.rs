use std::collections::HashMap;

use iced::{
    Subscription,
    keyboard::{self, Modifiers},
};

use crate::{
    app::Message,
    domain::{HistoryCommand, SceneMessage, Tool},
    infrastructure::IoProcess,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Binding {
    key: keyboard::Key,
    modifiers: keyboard::Modifiers,
}

#[derive(Debug, Clone)]
pub enum KeybindMessage {
    AddKeybind { key: Binding, message: Message },
    RemoveKeybind { key: Binding },
}

pub struct Keybinds {
    bindings: HashMap<Binding, Message>,
}

impl Keybinds {
    /// Creates a new keybindings with default keybinds set
    ///
    /// Undo:   Ctrl + Z
    /// Redo:   Ctrl + Shift + Z    Ctrl + Y
    ///
    /// Draw tool:  Ctrl + B
    /// Erase tool: Ctrl + E
    /// Pan Tool:   Ctrl + M
    ///
    /// Save:   Ctrl + S
    /// Open:   Ctrl + O
    pub fn new() -> Self {
        let bindings = HashMap::from([
            (
                Binding {
                    key: keyboard::Key::Character("z".into()),
                    modifiers: Modifiers::CTRL,
                },
                Message::History(HistoryCommand::Undo),
            ),
            (
                Binding {
                    key: keyboard::Key::Character("z".into()),
                    modifiers: Modifiers::CTRL | Modifiers::SHIFT,
                },
                Message::History(HistoryCommand::Redo),
            ),
            (
                Binding {
                    key: keyboard::Key::Character("y".into()),
                    modifiers: Modifiers::CTRL,
                },
                Message::History(HistoryCommand::Redo),
            ),
            (
                Binding {
                    key: keyboard::Key::Character("b".into()),
                    modifiers: Modifiers::CTRL,
                },
                Message::Scene(SceneMessage::ChangeTool(Tool::Paint)),
            ),
            (
                Binding {
                    key: keyboard::Key::Character("e".into()),
                    modifiers: Modifiers::CTRL,
                },
                Message::Scene(SceneMessage::ChangeTool(Tool::Erase)),
            ),
            (
                Binding {
                    key: keyboard::Key::Character("m".into()),
                    modifiers: Modifiers::CTRL,
                },
                Message::Scene(SceneMessage::ChangeTool(Tool::Pan)),
            ),
            (
                Binding {
                    key: keyboard::Key::Character("s".into()),
                    modifiers: Modifiers::CTRL,
                },
                Message::Save(IoProcess::Start),
            ),
            (
                Binding {
                    key: keyboard::Key::Character("o".into()),
                    modifiers: Modifiers::CTRL,
                },
                Message::Load(IoProcess::Start),
            ),
        ]);
        Self { bindings }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
        // keyboard::listen()
        // .filter_map(|event| match event {
        //     keyboard::Event::KeyPressed { key, modifiers, .. } => {
        //         Some(Binding { key, modifiers })
        //     }
        //     _ => None,
        // })
        // .with(self.bindings.clone())
        // .filter_map(|(bindings, binding)| bindings.get(&binding).cloned())
    }

    pub fn update(&mut self, message: KeybindMessage) {
        match message {
            KeybindMessage::AddKeybind { key, message } => self.bindings.insert(key, message),
            KeybindMessage::RemoveKeybind { key } => self.bindings.remove(&key),
        };
    }
}
