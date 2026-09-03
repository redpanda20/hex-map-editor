use crate::domain::{EditCommand, Scene};

#[derive(Debug, Default, Clone)]
pub struct History {
    undo_stack: Vec<Box<dyn EditCommand>>,
    redo_stack: Vec<Box<dyn EditCommand>>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Applies a fresh command
    pub fn apply(&mut self, scene: &mut Scene, command: Box<dyn EditCommand>) {
        #[cfg(debug_assertions)]
        let debug = format!("{command:?}");

        let inverse = command.apply(scene);
        if inverse.is_noop() {
            return;
        }

        #[cfg(debug_assertions)]
        println!("{debug}");

        self.redo_stack.clear();
        self.undo_stack.push(inverse);
    }

    /// Undo the last command.
    /// Returns if a command was applied or not.
    pub fn undo(&mut self, scene: &mut Scene) -> bool {
        let Some(command) = self.undo_stack.pop() else {
            return false;
        };
        #[cfg(debug_assertions)]
        println!("[Undo]: {command:?}");

        let redo = command.apply(scene);
        self.redo_stack.push(redo);
        true
    }

    /// Redo the last command
    /// Returns if a command was applied or not.
    pub fn redo(&mut self, scene: &mut Scene) -> bool {
        let Some(command) = self.redo_stack.pop() else {
            return false;
        };
        #[cfg(debug_assertions)]
        println!("[Redo]: {command:?}");

        let undo = command.apply(scene);
        self.undo_stack.push(undo);
        true
    }
}
