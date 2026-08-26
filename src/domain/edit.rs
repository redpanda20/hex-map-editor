use std::collections::HashSet;

use iced::{Color, Rectangle, widget::image::Handle};

use crate::domain::{
    HexCoord, LayerInner, LayerKind, Scene, flood_fill,
    id::{ImageId, LayerId},
    layer::{Layer, noise::NoiseParams},
};

/// A standalone edit that can be made to a scene
pub trait EditCommand: std::fmt::Debug + Send + EditCommandClone {
    /// Applies the edit, and returns its inverse
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand>;

    fn is_noop(&self) -> bool {
        false
    }
}

impl Clone for Box<dyn EditCommand> {
    fn clone(&self) -> Self {
        EditCommandClone::clone_box(&**self)
    }
}

pub trait EditCommandClone {
    fn clone_box(&self) -> Box<dyn EditCommand>;
}

impl<T> EditCommandClone for T
where
    T: 'static + EditCommand + Clone,
{
    fn clone_box(&self) -> Box<dyn EditCommand> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct NoOp;

#[derive(Debug, Clone)]
pub struct PushLayer {
    pub name: String,
    pub kind: LayerKind,
}

#[derive(Debug, Clone)]
pub struct InsertLayer {
    pub layer: Option<Layer>,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub struct RemoveLayer {
    pub id: LayerId,
}

#[derive(Debug, Clone)]
pub struct SwapLayers {
    pub id: LayerId,
    pub to: LayerId,
}

#[derive(Debug, Clone)]
pub struct MoveLayer {
    pub id: LayerId,
    pub to: usize,
}

#[derive(Debug, Clone)]
pub struct SetVisible {
    pub id: LayerId,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct Rename {
    pub id: LayerId,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PaintTile {
    pub layer: LayerId,
    pub coord: HexCoord,
}

#[derive(Debug, Clone)]
pub struct EraseTile {
    pub layer: LayerId,
    pub coord: HexCoord,
}

#[derive(Debug, Clone)]
pub struct PaintTiles {
    pub layer: LayerId,
    pub coords: HashSet<HexCoord>,
}

#[derive(Debug, Clone)]
pub struct EraseTiles {
    pub layer: LayerId,
    pub coords: HashSet<HexCoord>,
}

#[derive(Debug, Clone)]
pub struct BucketFill {
    pub layer: LayerId,
    pub from: HexCoord,
}

#[derive(Debug, Clone)]
pub struct InvertTiles {
    pub layer: LayerId,
}

#[derive(Debug, Clone)]
pub struct SetColour {
    pub layer: LayerId,
    pub colour: Color,
}

#[derive(Debug, Clone)]
pub struct SetNoiseSeed {
    pub layer: LayerId,
    pub seed: u64,
}

#[derive(Debug, Clone)]
pub struct SetNoiseParams {
    pub layer: LayerId,
    pub params: NoiseParams,
}

#[derive(Debug, Clone)]
pub struct SetImage {
    pub layer: LayerId,
    pub image: ImageId,
}

#[derive(Debug, Clone)]
pub struct SetImageBounds {
    pub layer: LayerId,
    pub bounds: Rectangle,
}

#[derive(Debug, Clone)]
pub struct SetImageOpacity {
    pub layer: LayerId,
    pub opacity: f32,
}

impl EditCommand for NoOp {
    fn apply(self: Box<Self>, _scene: &mut Scene) -> Box<dyn EditCommand> {
        self
    }
    fn is_noop(&self) -> bool {
        true
    }
}

impl EditCommand for PushLayer {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let kind = scene.new_kind(self.kind);
        let layer = Layer::new(self.name, kind);
        let id = layer.id;
        scene.insert_layer(layer, scene.inner.len());
        Box::new(RemoveLayer { id })
    }
}
impl EditCommand for InsertLayer {
    fn apply(mut self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let layer = self.layer.take().expect("InsertLayer applied twice");
        let id = layer.id;
        scene.insert_layer(layer, self.position);
        Box::new(RemoveLayer { id })
    }
}

impl EditCommand for RemoveLayer {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let (layer, position) = scene.remove_layer(self.id).expect("Layer not found");
        Box::new(InsertLayer {
            layer: Some(layer),
            position,
        })
    }
}

impl EditCommand for SwapLayers {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let to = scene
            .inner
            .iter()
            .position(|layer| layer.id == self.to)
            .expect("Layer `to` doesn't exist");
        let from = scene.move_layer(self.id, to).expect("Move failed");
        Box::new(MoveLayer {
            id: self.id,
            to: from,
        })
    }
}

impl EditCommand for MoveLayer {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let from = scene.move_layer(self.id, self.to).expect("Move failed");
        Box::new(MoveLayer {
            id: self.id,
            to: from,
        })
    }
}

impl EditCommand for SetVisible {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let layer = scene.get_layer_mut(self.id).expect("Layer not found");
        let previous = layer.visible;
        layer.visible = self.visible;

        Box::new(SetVisible {
            id: self.id,
            visible: previous,
        })
    }
}

impl EditCommand for Rename {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let layer = scene.get_layer_mut(self.id).expect("Layer not found");
        let previous = layer.name.clone();
        layer.name = self.name;

        Box::new(Rename {
            id: self.id,
            name: previous,
        })
    }
}

impl EditCommand for PaintTile {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        if scene.paint_tile(self.layer, self.coord) {
            Box::new(EraseTile {
                layer: self.layer,
                coord: self.coord,
            })
        } else {
            Box::new(NoOp)
        }
    }
}

impl EditCommand for EraseTile {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        if scene.erase_tile(self.layer, self.coord) {
            Box::new(PaintTile {
                layer: self.layer,
                coord: self.coord,
            })
        } else {
            Box::new(NoOp)
        }
    }
}

impl EditCommand for PaintTiles {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let layer = self.layer;
        let changes = scene.paint_tiles(layer, self.coords);
        if changes.is_empty() {
            return Box::new(NoOp);
        }
        Box::new(EraseTiles {
            layer,
            coords: changes,
        })
    }
}

impl EditCommand for EraseTiles {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let layer = self.layer;
        let changes = scene.erase_tiles(layer, self.coords);
        if changes.is_empty() {
            return Box::new(NoOp);
        }
        Box::new(PaintTiles {
            layer,
            coords: changes,
        })
    }
}

impl EditCommand for InvertTiles {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let Some(Layer {
            kind: LayerInner::Tiles(tiles),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        tiles.invert();

        Box::new(InvertTiles { layer: self.layer })
    }
}

impl EditCommand for BucketFill {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        // Make sure we are targeting a layer containing Tiles
        let Some(Layer {
            kind: LayerInner::Tiles(tiles),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        // If the flood fill overflows or underflows, invert the layer
        let Some(fill) = flood_fill(self.from, &tiles.tiles) else {
            tiles.invert();
            return Box::new(InvertTiles { layer: self.layer });
        };

        // Apply the changes to the tile layer, and make sure we actually applied changes
        let changes = scene.paint_tiles(self.layer, fill);
        if changes.is_empty() {
            return Box::new(NoOp);
        }
        Box::new(EraseTiles {
            layer: self.layer,
            coords: changes,
        })
    }
}

impl EditCommand for SetColour {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let Some(Layer {
            kind: LayerInner::Tiles(tiles),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        let prev = tiles.colour;
        tiles.colour = self.colour;

        Box::new(SetColour {
            layer: self.layer,
            colour: prev,
        })
    }
}

impl EditCommand for SetNoiseSeed {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let Some(Layer {
            kind: LayerInner::Perlin(noise),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        let seed = noise.get_seed();
        noise.set_seed(self.seed);

        Box::new(SetNoiseSeed {
            layer: self.layer,
            seed,
        })
    }
}

impl EditCommand for SetNoiseParams {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let Some(Layer {
            kind: LayerInner::Perlin(noise),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        let params = noise.get_params();
        noise.set_params(&self.params);

        Box::new(SetNoiseParams {
            layer: self.layer,
            params,
        })
    }
}

impl EditCommand for SetImage {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        // Get image size from store
        let (width, height) = {
            let Some(Handle::Rgba {
                id: _,
                width,
                height,
                pixels: _,
            }) = scene.assets.image_data(self.image)
            else {
                return Box::new(NoOp);
            };
            (width.cast_signed() as f32, height.cast_signed() as f32)
        };

        // Get mutable access to layer
        let Some(Layer {
            kind: LayerInner::Image(layer),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        layer.image = Some(self.image);
        layer.bounds.width = width;
        layer.bounds.height = height;

        // Irreversable edit
        // TODO: Make reversable
        Box::new(NoOp)
    }
}

impl EditCommand for SetImageBounds {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let Some(Layer {
            kind: LayerInner::Image(layer),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        let prev = layer.bounds;
        layer.bounds = self.bounds;

        Box::new(SetImageBounds {
            layer: self.layer,
            bounds: prev,
        })
    }
}

impl EditCommand for SetImageOpacity {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let Some(Layer {
            kind: LayerInner::Image(layer),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        let prev = layer.get_opacity();
        layer.set_opacity(self.opacity);

        Box::new(SetImageOpacity {
            layer: self.layer,
            opacity: prev,
        })
    }
}

// ------------------------- History ------------------------------
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
        let inverse = command.apply(scene);
        if inverse.is_noop() {
            return;
        }
        self.redo_stack.clear();
        self.undo_stack.push(inverse);
    }

    /// Undo the last command.
    /// Returns if a command was applied or not.
    pub fn undo(&mut self, scene: &mut Scene) -> bool {
        let Some(command) = self.undo_stack.pop() else {
            return false;
        };
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
        let undo = command.apply(scene);
        self.undo_stack.push(undo);
        true
    }
}
