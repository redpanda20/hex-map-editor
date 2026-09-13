pub mod assets;
pub mod colour;
pub mod edit;
mod hex;
pub mod history;
pub mod id;
pub mod layer;
mod scene;
mod tool;

mod ports;

pub use hex::{HexBounds, HexCoord, flood_fill};

pub use scene::Scene;

pub use tool::Tool;

pub use edit::EditCommand;
pub use history::History;
pub use layer::{Layer, LayerInner, LayerKind};
pub use ports::RenderTarget;

/// End-to-end workflow tests that exercise `History` + `Scene` +
/// `EditCommand` together, the way `App::update` does in practice, rather
/// than any one command in isolation.
///
/// (These live here instead of under `tests/` because this crate only has
/// a binary target - there's no `src/lib.rs` for an external integration
/// test to link against. See the accompanying notes for more on this.)
#[cfg(test)]
mod workflow_tests {
    use crate::domain::{
        HexCoord, History, LayerInner, LayerKind, Scene,
        edit::{BucketFill, PaintTiles, PushLayer, SetVisible},
        id::LayerId,
    };

    fn is_painted(scene: &Scene, layer: LayerId, coord: HexCoord) -> bool {
        match &scene.get_layer(layer).unwrap().kind {
            LayerInner::Tiles(tiles) => tiles.tiles.contains(&coord),
            _ => panic!("expected a Tiles layer"),
        }
    }

    #[test]
    fn paint_a_shape_undo_it_then_redo_it() {
        let mut scene = Scene::default();
        let mut history = History::new();
        let layer = scene.inner[0].id;

        let coords: Vec<HexCoord> = HexCoord { col: 0, row: 0 }
            .neighbors()
            .into_iter()
            .chain(std::iter::once(HexCoord { col: 0, row: 0 }))
            .collect();

        history.apply(
            &mut scene,
            Box::new(PaintTiles {
                layer,
                coords: coords.clone(),
            }),
        );
        for coord in &coords {
            assert!(is_painted(&scene, layer, *coord));
        }

        history.undo(&mut scene);
        for coord in &coords {
            assert!(!is_painted(&scene, layer, *coord));
        }

        history.redo(&mut scene);
        for coord in &coords {
            assert!(is_painted(&scene, layer, *coord));
        }
    }

    #[test]
    fn add_a_layer_paint_on_it_bucket_fill_and_undo_everything() {
        let mut scene = Scene::default();
        let mut history = History::new();
        let starting_layer_count = scene.inner.len();

        history.apply(
            &mut scene,
            Box::new(PushLayer {
                name: "Terrain".into(),
                kind: LayerKind::Tiles,
            }),
        );
        let new_layer = scene.inner.last().unwrap().id;

        let ring: Vec<HexCoord> = HexCoord { col: 0, row: 0 }
            .neighbors()
            .into_iter()
            .collect();
        history.apply(
            &mut scene,
            Box::new(PaintTiles {
                layer: new_layer,
                coords: ring,
            }),
        );

        history.apply(
            &mut scene,
            Box::new(BucketFill {
                layer: new_layer,
                from: HexCoord { col: 0, row: 0 },
            }),
        );
        assert!(is_painted(&scene, new_layer, HexCoord { col: 0, row: 0 }));

        history.apply(
            &mut scene,
            Box::new(SetVisible {
                id: new_layer,
                visible: false,
            }),
        );
        assert!(!scene.get_layer(new_layer).unwrap().visible);

        // Unwind the whole session.
        while history.can_undo() {
            history.undo(&mut scene);
        }

        assert_eq!(scene.inner.len(), starting_layer_count);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }
}
