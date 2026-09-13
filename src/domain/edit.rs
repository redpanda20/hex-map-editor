use iced::{Color, Point, Size, widget::image::Handle};

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
pub struct MoveLayerTo {
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
    pub coords: Vec<HexCoord>,
}

#[derive(Debug, Clone)]
pub struct EraseTiles {
    pub layer: LayerId,
    pub coords: Vec<HexCoord>,
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
pub struct SetImageAndSize {
    pub layer: LayerId,
    pub image: Option<ImageId>,
    pub size: Size,
}

#[derive(Debug, Clone)]
pub struct SetImage {
    pub layer: LayerId,
    pub image: ImageId,
}

#[derive(Debug, Clone)]
pub struct SetImageSize {
    pub layer: LayerId,
    pub size: Size,
}

#[derive(Debug, Clone)]
pub struct SetImagePosition {
    pub layer: LayerId,
    pub position: Point,
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

impl EditCommand for MoveLayerTo {
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
        if self.visible == layer.visible {
            return Box::new(NoOp);
        }

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
        if self.name == layer.name {
            return Box::new(NoOp);
        }

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
        if self.colour == tiles.colour {
            return Box::new(NoOp);
        }

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
        if self.seed == noise.get_seed() {
            return Box::new(NoOp);
        }

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
        if self.params == noise.get_params() {
            return Box::new(NoOp);
        }

        let params = noise.get_params();
        noise.set_params(&self.params);

        Box::new(SetNoiseParams {
            layer: self.layer,
            params,
        })
    }
}

impl EditCommand for SetImageAndSize {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        // Get mutable access to layer
        let Some(Layer {
            kind: LayerInner::Image(layer),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        let prev_image = layer.image;
        let prev_size = layer.size;

        layer.size = self.size;
        layer.image = self.image;

        Box::new(SetImageAndSize {
            layer: self.layer,
            image: prev_image,
            size: prev_size,
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

        let prev_image = layer.image;
        let prev_size = layer.size;

        layer.image = Some(self.image);
        layer.size = Size { width, height };

        Box::new(SetImageAndSize {
            layer: self.layer,
            image: prev_image,
            size: prev_size,
        })
    }
}

impl EditCommand for SetImageSize {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let Some(Layer {
            kind: LayerInner::Image(layer),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        if self.size == layer.size {
            return Box::new(NoOp);
        }

        let prev = layer.size;
        layer.size = self.size;

        Box::new(SetImageSize {
            layer: self.layer,
            size: prev,
        })
    }
}

impl EditCommand for SetImagePosition {
    fn apply(self: Box<Self>, scene: &mut Scene) -> Box<dyn EditCommand> {
        let Some(Layer {
            kind: LayerInner::Image(layer),
            ..
        }) = scene.get_layer_mut(self.layer)
        else {
            return Box::new(NoOp);
        };

        if self.position == layer.position {
            return Box::new(NoOp);
        }

        let prev = layer.position;
        layer.position = self.position;

        Box::new(SetImagePosition {
            layer: self.layer,
            position: prev,
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
        if self.opacity == layer.get_opacity() {
            return Box::new(NoOp);
        }

        let prev = layer.get_opacity();
        layer.set_opacity(self.opacity);

        Box::new(SetImageOpacity {
            layer: self.layer,
            opacity: prev,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use iced::{Color, Point, Size};

    use super::*;
    use crate::domain::assets::ImageAsset;

    /// Applies a given command then applies the inverse.
    /// This property should hold for all (valid) cases of `EditCommand`.
    fn assert_apply_then_undo_is_identity(scene: &mut Scene, command: Box<dyn EditCommand>) {
        // Only the layers of the `Scene` are compared, as revision counter should always break equality.
        let before = format!("{:?}", scene.inner);
        let inverse = command.apply(scene);
        assert_ne!(
            format!("{:?}", scene.inner),
            before,
            "command should have changed the scene"
        );
        inverse.apply(scene);
        assert_eq!(
            format!("{:?}", scene.inner),
            before,
            "applying the inverse should restore the original scene"
        );
    }

    fn scene_with_tiles_layer() -> (Scene, LayerId) {
        let scene = Scene::default();
        let id = scene.inner[0].id;
        (scene, id)
    }

    fn scene_with_noise_layer() -> (Scene, LayerId) {
        let mut scene = Scene::default();
        let layer = Layer::new("Noise", scene.new_kind(LayerKind::Noise));
        let id = layer.id;
        scene.insert_layer(layer, scene.inner.len());
        (scene, id)
    }

    fn scene_with_image_layer() -> (Scene, LayerId) {
        let mut scene = Scene::default();
        let layer = Layer::new("Image", scene.new_kind(LayerKind::Image));
        let id = layer.id;
        scene.insert_layer(layer, scene.inner.len());
        (scene, id)
    }

    fn register_test_image(scene: &mut Scene, width: u32, height: u32) -> ImageId {
        let pixel_count = (width * height * 4) as usize;
        scene.assets.register_image(ImageAsset {
            encoded: vec![],
            extension: "png".into(),
            data: vec![0u8; pixel_count],
            width,
            height,
            name: "test.png".into(),
        })
    }

    // ---- NoOp ----

    #[test]
    fn noop_is_a_noop_and_applies_to_itself() {
        let mut scene = Scene::default();
        let before = format!("{scene:?}");
        let inverse = Box::new(NoOp).apply(&mut scene);
        assert!(inverse.is_noop());
        assert_eq!(format!("{scene:?}"), before);
    }

    // ----  PushLayer / InsertLayer / RemoveLayer ----

    #[test]
    fn push_layer_adds_a_layer_and_undoes_cleanly() {
        let mut scene = Scene::default();
        let before_len = scene.inner.len();

        let command = Box::new(PushLayer {
            name: "New Layer".into(),
            kind: LayerKind::Tiles,
        });
        let inverse = command.apply(&mut scene);
        assert_eq!(scene.inner.len(), before_len + 1);
        assert_eq!(scene.inner.last().unwrap().name, "New Layer");

        inverse.apply(&mut scene);
        assert_eq!(scene.inner.len(), before_len);
    }

    #[test]
    fn remove_then_insert_layer_round_trips() {
        let (mut scene, id) = scene_with_tiles_layer();
        assert_apply_then_undo_is_identity(&mut scene, Box::new(RemoveLayer { id }));
    }

    #[test]
    #[should_panic(expected = "Layer not found")]
    fn remove_layer_of_missing_id_panics() {
        let mut scene = Scene::default();
        Box::new(RemoveLayer {
            id: LayerId::from_raw(u64::MAX),
        })
        .apply(&mut scene);
    }

    // ---- MoveLayerTo / MoveLayer ----

    #[test]
    fn move_layer_to_round_trips() {
        let mut scene = Scene::default();
        let a = scene.inner[0].id;
        let b = Layer::new("B", scene.new_kind(LayerKind::Tiles));
        let b_id = b.id;
        scene.insert_layer(b, 1);

        assert_apply_then_undo_is_identity(&mut scene, Box::new(MoveLayerTo { id: a, to: b_id }));
    }

    // ---- SetVisible / Rename ----

    #[test]
    fn set_visible_toggles_and_undoes() {
        let (mut scene, id) = scene_with_tiles_layer();
        assert!(scene.get_layer(id).unwrap().visible);

        assert_apply_then_undo_is_identity(&mut scene, Box::new(SetVisible { id, visible: false }));
    }

    #[test]
    fn set_visible_to_the_current_value_is_a_noop() {
        let (mut scene, id) = scene_with_tiles_layer();
        let inverse = Box::new(SetVisible { id, visible: true }).apply(&mut scene);
        assert!(inverse.is_noop());
    }

    #[test]
    fn rename_round_trips_and_noops_on_identical_name() {
        let (mut scene, id) = scene_with_tiles_layer();
        assert_apply_then_undo_is_identity(
            &mut scene,
            Box::new(Rename {
                id,
                name: "Renamed".into(),
            }),
        );

        let current_name = scene.get_layer(id).unwrap().name.clone();
        let inverse = Box::new(Rename {
            id,
            name: current_name,
        })
        .apply(&mut scene);
        assert!(inverse.is_noop());
    }

    // ---- PaintTile / EraseTile / PaintTiles / EraseTiles ----

    #[test]
    fn paint_tile_round_trips_and_noops_when_already_painted() {
        let (mut scene, id) = scene_with_tiles_layer();
        let coord = HexCoord { col: 0, row: 0 };

        assert_apply_then_undo_is_identity(&mut scene, Box::new(PaintTile { layer: id, coord }));

        scene.paint_tile(id, coord);
        let inverse = Box::new(PaintTile { layer: id, coord }).apply(&mut scene);
        assert!(inverse.is_noop());
    }

    #[test]
    fn paint_tiles_round_trips() {
        let (mut scene, id) = scene_with_tiles_layer();
        let coords = vec![
            HexCoord { col: 0, row: 0 },
            HexCoord { col: 1, row: 0 },
            HexCoord { col: 0, row: 1 },
        ];
        assert_apply_then_undo_is_identity(&mut scene, Box::new(PaintTiles { layer: id, coords }));
    }

    #[test]
    fn paint_tiles_with_no_new_tiles_is_a_noop() {
        let (mut scene, id) = scene_with_tiles_layer();
        let coord = HexCoord { col: 0, row: 0 };
        scene.paint_tile(id, coord);

        let inverse = Box::new(PaintTiles {
            layer: id,
            coords: vec![coord],
        })
        .apply(&mut scene);
        assert!(inverse.is_noop());
    }

    #[test]
    fn erase_tiles_round_trips() {
        let (mut scene, id) = scene_with_tiles_layer();
        let coords = vec![HexCoord { col: 0, row: 0 }, HexCoord { col: 1, row: 0 }];
        scene.paint_tiles(id, coords.clone());

        assert_apply_then_undo_is_identity(&mut scene, Box::new(EraseTiles { layer: id, coords }));
    }

    #[test]
    fn erase_tile_noops_when_already_empty() {
        let (mut scene, id) = scene_with_tiles_layer();
        let coord = HexCoord { col: 0, row: 0 };

        let inverse = Box::new(EraseTile { layer: id, coord }).apply(&mut scene);

        assert!(inverse.is_noop());
    }

    // ---- InvertTiles ----

    #[test]
    fn invert_tiles_is_its_own_inverse() {
        let (mut scene, id) = scene_with_tiles_layer();
        assert_apply_then_undo_is_identity(&mut scene, Box::new(InvertTiles { layer: id }));
    }

    #[test]
    fn invert_tiles_on_non_tiles_layer_is_a_noop() {
        let (mut scene, id) = scene_with_noise_layer();
        let inverse = Box::new(InvertTiles { layer: id }).apply(&mut scene);
        assert!(inverse.is_noop());
    }

    // ---- BucketFill ----

    #[test]
    fn bucket_fill_paints_a_bounded_empty_region_and_undoes() {
        // NOTE: this deliberately does *not* use
        // `assert_apply_then_undo_is_identity`, because it compares scenes
        // via `Debug`, and `HashSet`'s `Debug` order depends on its
        // internal table capacity - which can grow (and won't shrink back)
        // partway through this test. Comparing the `HashSet`s directly
        // sidesteps that: `HashSet: PartialEq` is defined by set membership,
        // not iteration order.
        fn tile_set(scene: &Scene, id: LayerId) -> HashSet<HexCoord> {
            let Some(Layer {
                kind: LayerInner::Tiles(tiles),
                ..
            }) = scene.get_layer(id)
            else {
                panic!("expected tiles layer");
            };
            tiles.tiles.clone()
        }

        let (mut scene, id) = scene_with_tiles_layer();
        // Paint a ring, leaving (0,0) as a bounded empty "hole".
        let ring: Vec<HexCoord> = HexCoord { col: 0, row: 0 }.neighbors().to_vec();
        scene.paint_tiles(id, ring);
        let before = tile_set(&scene, id);
        assert!(!before.contains(&HexCoord { col: 0, row: 0 }));

        let inverse = Box::new(BucketFill {
            layer: id,
            from: HexCoord { col: 0, row: 0 },
        })
        .apply(&mut scene);

        let after_fill = tile_set(&scene, id);
        assert!(after_fill.contains(&HexCoord { col: 0, row: 0 }));
        assert_eq!(after_fill.len(), before.len() + 1);

        inverse.apply(&mut scene);
        assert_eq!(
            tile_set(&scene, id),
            before,
            "undo should restore the original tiles"
        );
    }

    #[test]
    fn bucket_fill_of_an_unbounded_region_inverts_the_layer_instead() {
        let (mut scene, id) = scene_with_tiles_layer();
        // A single, distant painted tile: filling from empty space has no
        // bounding box to contain it, so the whole layer is inverted.
        scene.paint_tile(id, HexCoord { col: 0, row: 0 });

        let inverse = Box::new(BucketFill {
            layer: id,
            from: HexCoord { col: 50, row: 50 },
        })
        .apply(&mut scene);

        let Some(Layer {
            kind: LayerInner::Tiles(tiles),
            ..
        }) = scene.get_layer(id)
        else {
            panic!("expected tiles layer");
        };
        assert!(tiles.is_inverted());

        // The inverse is itself another InvertTiles, so re-applying it
        // should flip the layer back to normal.
        inverse.apply(&mut scene);
        let Some(Layer {
            kind: LayerInner::Tiles(tiles),
            ..
        }) = scene.get_layer(id)
        else {
            panic!("expected tiles layer");
        };
        assert!(!tiles.is_inverted());
    }

    #[test]
    fn bucket_fill_on_non_tiles_layer_is_a_noop() {
        let (mut scene, id) = scene_with_noise_layer();
        let inverse = Box::new(BucketFill {
            layer: id,
            from: HexCoord { col: 0, row: 0 },
        })
        .apply(&mut scene);
        assert!(inverse.is_noop());
    }

    // ---- SetColour ----

    #[test]
    fn set_colour_round_trips_and_noops_on_identical_colour() {
        let (mut scene, id) = scene_with_tiles_layer();
        assert_apply_then_undo_is_identity(
            &mut scene,
            Box::new(SetColour {
                layer: id,
                colour: Color::from_rgb(0.1, 0.2, 0.3),
            }),
        );

        let current_colour = {
            let Some(Layer {
                kind: LayerInner::Tiles(tiles),
                ..
            }) = scene.get_layer(id)
            else {
                panic!("expected tiles layer");
            };
            tiles.colour
        };
        let inverse = Box::new(SetColour {
            layer: id,
            colour: current_colour,
        })
        .apply(&mut scene);
        assert!(inverse.is_noop());
    }

    #[test]
    fn set_colour_on_non_tiles_layer_is_a_noop() {
        let (mut scene, id) = scene_with_noise_layer();
        let inverse = Box::new(SetColour {
            layer: id,
            colour: Color::BLACK,
        })
        .apply(&mut scene);
        assert!(inverse.is_noop());
    }

    // ---- SetNoiseSeed / SetNoiseParams ----

    #[test]
    fn set_noise_seed_round_trips_and_noops_on_identical_seed() {
        let (mut scene, id) = scene_with_noise_layer();
        let current_seed = {
            let Some(Layer {
                kind: LayerInner::Perlin(noise),
                ..
            }) = scene.get_layer(id)
            else {
                panic!("expected noise layer");
            };
            noise.get_seed()
        };

        assert_apply_then_undo_is_identity(
            &mut scene,
            Box::new(SetNoiseSeed {
                layer: id,
                seed: current_seed.wrapping_add(1),
            }),
        );

        let inverse = Box::new(SetNoiseSeed {
            layer: id,
            seed: current_seed,
        })
        .apply(&mut scene);
        assert!(inverse.is_noop());
    }

    #[test]
    fn set_noise_seed_on_non_noise_layer_is_a_noop() {
        let (mut scene, id) = scene_with_tiles_layer();
        let inverse = Box::new(SetNoiseSeed {
            layer: id,
            seed: 42,
        })
        .apply(&mut scene);
        assert!(inverse.is_noop());
    }

    #[test]
    fn set_noise_params_round_trips() {
        let (mut scene, id) = scene_with_noise_layer();
        let new_params = NoiseParams {
            threshold: 0.5,
            frequency: 10.0,
            octaves: 3,
            persistence: 0.25,
        };
        assert_apply_then_undo_is_identity(
            &mut scene,
            Box::new(SetNoiseParams {
                layer: id,
                params: new_params,
            }),
        );
    }

    // -- SetImageAndSize / SetImage / SetImageBounds / SetImageOpacity --

    #[test]
    fn set_image_and_size_round_trips() {
        let (mut scene, id) = scene_with_image_layer();
        let image_id = register_test_image(&mut scene, 4, 4);

        assert_apply_then_undo_is_identity(
            &mut scene,
            Box::new(SetImageAndSize {
                layer: id,
                image: Some(image_id),
                size: Size::new(40.0, 40.0),
            }),
        );
    }

    #[test]
    fn set_image_looks_up_dimensions_from_the_asset_store() {
        let (mut scene, id) = scene_with_image_layer();
        let image_id = register_test_image(&mut scene, 8, 6);

        Box::new(SetImage {
            layer: id,
            image: image_id,
        })
        .apply(&mut scene);

        let Some(Layer {
            kind: LayerInner::Image(image_layer),
            ..
        }) = scene.get_layer(id)
        else {
            panic!("expected image layer");
        };
        assert_eq!(image_layer.image, Some(image_id));
        assert_eq!(image_layer.size.width, 8.0);
        assert_eq!(image_layer.size.height, 6.0);
    }

    #[test]
    fn set_image_of_unregistered_asset_is_a_noop() {
        let (mut scene, id) = scene_with_image_layer();
        let inverse = Box::new(SetImage {
            layer: id,
            image: ImageId::from_raw(u64::MAX),
        })
        .apply(&mut scene);
        assert!(inverse.is_noop());
    }

    #[test]
    fn set_image_size_round_trips() {
        let (mut scene, id) = scene_with_image_layer();
        // let bounds = Rectangle::new(Point::new(1.0, 2.0), Size::new(30.0, 40.0));
        let size = Size::new(30.0, 40.0);

        assert_apply_then_undo_is_identity(&mut scene, Box::new(SetImageSize { layer: id, size }));
    }

    #[test]
    fn set_image_size_noops_on_identical_bounds() {
        let (mut scene, id) = scene_with_image_layer();
        // let bounds = Rectangle::new(Point::new(1.0, 2.0), Size::new(30.0, 40.0));
        let size = Size::new(30.0, 40.0);

        let _ = Box::new(SetImageSize { layer: id, size }).apply(&mut scene);
        let inverse = Box::new(SetImageSize { layer: id, size }).apply(&mut scene);

        assert!(inverse.is_noop());
    }

    #[test]
    fn set_image_position_round_trips() {
        let (mut scene, id) = scene_with_image_layer();
        let position = Point::new(1.0, 2.0);

        assert_apply_then_undo_is_identity(
            &mut scene,
            Box::new(SetImagePosition {
                layer: id,
                position,
            }),
        );
    }

    #[test]
    fn set_image_position_noops_on_identical_bounds() {
        let (mut scene, id) = scene_with_image_layer();
        let position = Point::new(1.0, 2.0);

        let _ = Box::new(SetImagePosition {
            layer: id,
            position,
        })
        .apply(&mut scene);
        let inverse = Box::new(SetImagePosition {
            layer: id,
            position,
        })
        .apply(&mut scene);

        assert!(inverse.is_noop());
    }

    #[test]
    fn set_image_opacity_round_trips_and_clamps() {
        let (mut scene, id) = scene_with_image_layer();

        assert_apply_then_undo_is_identity(
            &mut scene,
            Box::new(SetImageOpacity {
                layer: id,
                opacity: 0.4,
            }),
        );

        // Over-range opacity should be clamped to 1.0 when applied.
        Box::new(SetImageOpacity {
            layer: id,
            opacity: 5.0,
        })
        .apply(&mut scene);
        let Some(Layer {
            kind: LayerInner::Image(image_layer),
            ..
        }) = scene.get_layer(id)
        else {
            panic!("expected image layer");
        };
        assert_eq!(image_layer.get_opacity(), 1.0);
    }

    #[test]
    fn set_image_opacity_on_non_image_layer_is_a_noop() {
        let (mut scene, id) = scene_with_tiles_layer();
        let inverse = Box::new(SetImageOpacity {
            layer: id,
            opacity: 0.5,
        })
        .apply(&mut scene);
        assert!(inverse.is_noop());
    }
}
