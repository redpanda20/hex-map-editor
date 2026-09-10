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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::edit::{PaintTile, PushLayer, Rename};
    use crate::domain::{HexCoord, LayerKind};

    #[test]
    fn fresh_history_cannot_undo_or_redo() {
        let history = History::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn apply_pushes_onto_the_undo_stack() {
        let mut scene = Scene::default();
        let mut history = History::new();

        history.apply(
            &mut scene,
            Box::new(PushLayer {
                name: "New".into(),
                kind: LayerKind::Tiles,
            }),
        );

        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn apply_of_a_noop_command_is_not_recorded() {
        let mut scene = Scene::default();
        let id = scene.inner[0].id;
        let mut history = History::new();

        // Renaming to the layer's current name is a no-op (see edit.rs).
        let current_name = scene.get_layer(id).unwrap().name.clone();
        history.apply(
            &mut scene,
            Box::new(Rename {
                id,
                name: current_name,
            }),
        );

        assert!(!history.can_undo(), "a no-op edit shouldn't be undoable");
    }

    #[test]
    fn undo_then_redo_restores_the_scene_each_time() {
        let mut scene = Scene::default();
        let id = scene.inner[0].id;
        let mut history = History::new();
        let coord = HexCoord { col: 0, row: 0 };

        history.apply(&mut scene, Box::new(PaintTile { layer: id, coord }));
        assert!(scene.get_layer(id).is_some());
        let painted = format!("{:?}", scene.inner);

        assert!(history.undo(&mut scene));
        let unpainted = format!("{:?}", scene.inner);
        assert_ne!(painted, unpainted);
        assert!(history.can_redo());

        assert!(history.redo(&mut scene));
        assert_eq!(format!("{:?}", scene.inner), painted);
        assert!(!history.can_redo());
    }

    #[test]
    fn undo_and_redo_on_empty_stacks_report_false() {
        let mut scene = Scene::default();
        let mut history = History::new();
        assert!(!history.undo(&mut scene));
        assert!(!history.redo(&mut scene));
    }

    #[test]
    fn a_fresh_apply_clears_the_redo_stack() {
        let mut scene = Scene::default();
        let id = scene.inner[0].id;
        let mut history = History::new();

        history.apply(
            &mut scene,
            Box::new(PaintTile {
                layer: id,
                coord: HexCoord { col: 0, row: 0 },
            }),
        );
        history.undo(&mut scene);
        assert!(history.can_redo());

        history.apply(
            &mut scene,
            Box::new(PaintTile {
                layer: id,
                coord: HexCoord { col: 1, row: 0 },
            }),
        );
        assert!(
            !history.can_redo(),
            "a new edit should invalidate the old redo history"
        );
    }

    #[test]
    fn undo_redo_sequence_across_multiple_edits() {
        let mut scene = Scene::default();
        let id = scene.inner[0].id;
        let mut history = History::new();

        for col in 0..3 {
            history.apply(
                &mut scene,
                Box::new(PaintTile {
                    layer: id,
                    coord: HexCoord { col, row: 0 },
                }),
            );
        }

        history.undo(&mut scene);
        history.undo(&mut scene);
        history.undo(&mut scene);
        assert!(!history.can_undo());

        for _ in 0..3 {
            assert!(history.redo(&mut scene));
        }
        assert!(!history.can_redo());
    }
}
