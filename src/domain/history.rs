use iced::{Color, Task};

use crate::{
    app::Message,
    domain::{HexCoord, Layer, NoiseOctaves, Scene, SceneMessage},
};

#[derive(Debug, Clone)]
pub enum HistoryCommand {
    Undo,
    Redo,
    BeginTransaction(SceneMessage),
    CommitTransaction,
}

/// Records every content changing `SceneCommand` as a series of edits.
pub struct History {
    pub undo_stack: Vec<Edit>,
    pub redo_stack: Vec<Edit>,

    /// Used to coalesce many `apply` calls into a single undo step.
    /// e.g. every hex touched by one mouse drag stroke
    open_transaction: Option<Vec<Edit>>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            open_transaction: None,
        }
    }

    /// Include an edit in history
    ///
    /// Pushes change to transaction if available,
    /// otherwise pushes change directly to undo history
    pub fn apply(&mut self, edit: Option<Edit>) {
        let Some(edit) = edit else { return };

        match &mut self.open_transaction {
            Some(batch) => batch.push(edit),
            None => self.undo_stack.push(edit),
        }
    }

    pub fn update(&mut self, command: HistoryCommand, scene: &mut Scene) -> Task<Message> {
        match command {
            HistoryCommand::Undo => _ = self.undo(scene),
            HistoryCommand::Redo => _ = self.redo(scene),
            HistoryCommand::BeginTransaction(scene_command) => {
                self.begin_transaction();
                return Task::done(Message::Scene(scene_command));
            }
            HistoryCommand::CommitTransaction => self.commit_transaction(),
        }

        Task::none()
    }

    /// Opens a transaction
    ///
    /// Repeated calls just extend the existing transaction.
    /// Intended for continuous gestures like a drag-painted stroke.
    fn begin_transaction(&mut self) {
        if self.open_transaction.is_none() {
            self.open_transaction = Some(Vec::new());
        }
    }

    /// Closes an open transaction
    ///
    /// A transaction that produced no edits pushes nothing,
    /// so an empty drag doesn't create a no-op undo entry.
    fn commit_transaction(&mut self) {
        if let Some(batch) = self.open_transaction.take() {
            if let Some(edit) = Edit::coalesce(batch) {
                self.undo_stack.push(edit);
                self.redo_stack.clear();
            }
        }
    }

    fn undo(&mut self, scene: &mut Scene) -> bool {
        let Some(batch) = self.undo_stack.pop() else {
            return false;
        };
        match &batch {
            Edit::Batch { edits } => edits.iter().rev().for_each(|e| scene.undo_edit(e)),
            edit => scene.undo_edit(edit),
        };
        self.redo_stack.push(batch);
        true
    }

    fn redo(&mut self, scene: &mut Scene) -> bool {
        let Some(batch) = self.redo_stack.pop() else {
            return false;
        };
        match &batch {
            Edit::Batch { edits } => edits.iter().rev().for_each(|e| scene.redo_edit(e)),
            edit => scene.redo_edit(edit),
        }
        self.undo_stack.push(batch);
        true
    }
}

/// An invertible change to a scene
#[derive(Debug, Clone)]
pub enum Edit {
    Batch {
        edits: Vec<Edit>,
    },
    Tile {
        layer: usize,
        coord: HexCoord,
        before: bool,
        after: bool,
    },

    LayerAdded {
        index: usize,
        layer: Layer,
    },
    LayerRemoved {
        index: usize,
        layer: Layer,
    },
    LayersSwapped {
        a: usize,
        b: usize,
    },
    /// Flip between `LayerInner::Tiles` <-> `LayerInner::InvertedTiles`. Self-inverse.
    LayerInverted {
        index: usize,
    },

    LayerVisibility {
        index: usize,
        before: bool,
        after: bool,
    },
    LayerName {
        index: usize,
        before: String,
        after: String,
    },
    LayerColour {
        index: usize,
        before: Color,
        after: Color,
    },

    LayerSeed {
        index: usize,
        before: u64,
        after: u64,
    },
    LayerScale {
        index: usize,
        before: f32,
        after: f32,
    },
    LayerThreshold {
        index: usize,
        before: f32,
        after: f32,
    },
    LayerOctaves {
        index: usize,
        before: NoiseOctaves,
        after: NoiseOctaves,
    },
    LayerPersistence {
        index: usize,
        before: f32,
        after: f32,
    },
}

impl Edit {
    /// Collapse a vec of edits into a singular edit.
    ///
    /// None if vec was empty
    /// Edit if there was a singular change
    /// Edit::Batch { edits } otherwise
    pub fn coalesce(edits: Vec<Edit>) -> Option<Edit> {
        match edits.len() {
            0 => None,
            1 => edits.into_iter().next(),
            _ => Some(Edit::Batch { edits }),
        }
    }
}
