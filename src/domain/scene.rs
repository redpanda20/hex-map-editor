use iced::Color;
use rand::random;

use crate::domain::{
    HexCoord, Layer, LayerInner, LayerKind,
    assets::AssetStore,
    id::LayerId,
    layer::{LayerInnerImpl, image::ImageLayer, noise::PerlinNoiseLayer, tiles::SparseTiles},
};

const DEFAULT_COLORS: [Color; 5] = [
    Color::from_rgba8(245, 196, 168, 0.9),
    Color::from_rgba8(168, 212, 176, 0.9),
    Color::from_rgba8(168, 200, 245, 0.9),
    Color::from_rgba8(196, 168, 245, 0.9),
    Color::from_rgba8(245, 168, 200, 0.9),
];

#[derive(Debug, Clone)]
pub struct Scene {
    pub inner: Vec<Layer>,
    pub assets: AssetStore,
    revision: u64,
}

impl Scene {
    /// Load a scene with a existing Layers & AssetStore.
    pub fn from_layers_with_assets(inner: Vec<Layer>, assets: AssetStore) -> Self {
        Self {
            inner,
            revision: 1,
            assets,
        }
    }
}

impl Scene {
    /// A counter incremented when the state of Layers changes.
    /// Used by consumers to invalidate out of date caches
    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn change_revision(&mut self) {
        self.revision = self.revision().wrapping_add(1);
    }

    pub fn new_kind(&self, kind: LayerKind) -> LayerInner {
        match kind {
            LayerKind::Tiles => {
                LayerInner::Tiles(SparseTiles::new(DEFAULT_COLORS[self.inner.len() % 5]))
            }
            LayerKind::Noise => LayerInner::Perlin(PerlinNoiseLayer::new(random())),
            LayerKind::Image => LayerInner::Image(ImageLayer::new()),
        }
    }

    pub fn get_visible_layers(&self) -> Vec<&dyn LayerInnerImpl> {
        self.inner
            .iter()
            .filter(|layer| layer.visible)
            .map(|layer| &layer.kind as &dyn LayerInnerImpl)
            .collect()
    }

    /// Insert a new layer at a given index
    pub fn insert_layer(&mut self, layer: Layer, index: usize) -> Option<()> {
        self.inner.insert(index, layer);
        self.change_revision();
        Some(())
    }

    /// Remove a layer with a given LayerId
    pub fn remove_layer(&mut self, id: LayerId) -> Option<(Layer, usize)> {
        let index = self.inner.iter().position(|layer| layer.id == id)?;
        self.change_revision();
        Some((self.inner.remove(index), index))
    }

    /// Move a layer with a given LayerId, to a new position
    pub fn move_layer(&mut self, id: LayerId, to: usize) -> Option<usize> {
        let index = self.inner.iter().position(|layer| layer.id == id)?;
        self.change_revision();
        self.inner.swap(index, to);
        Some(index)
    }

    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        let index = self.inner.iter().position(|layer| layer.id == id)?;
        self.inner.get(index)
    }

    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        let index = self.inner.iter().position(|layer| layer.id == id)?;
        self.change_revision();
        self.inner.get_mut(index)
    }

    /// Paint a tile on the layer with `id` at tile `coord`.
    /// Returns if a change was made, false if the tile was already filled,
    /// is an incompatible layer, or doesn't exist
    pub fn paint_tile(&mut self, id: LayerId, coord: HexCoord) -> bool {
        let result = match self
            .inner
            .iter_mut()
            .find(|layer| layer.id == id)
            .map(|layer| &mut layer.kind)
        {
            Some(LayerInner::Tiles(t)) => t.paint(coord),
            _ => false,
        };
        if result {
            self.change_revision();
        }
        result
    }

    /// Erase the tile at `coord` on a given layer
    /// Returns if a change was made, false if the tile was already filled,
    /// is an incompatible layer, or doesn't exist
    pub fn erase_tile(&mut self, id: LayerId, coord: HexCoord) -> bool {
        let result = match self
            .inner
            .iter_mut()
            .find(|layer| layer.id == id)
            .map(|layer| &mut layer.kind)
        {
            Some(LayerInner::Tiles(t)) => t.erase(coord),
            _ => false,
        };
        if result {
            self.change_revision();
        }
        result
    }

    /// Paints multiple tiles in once pass on a given layer
    /// Returns all tiles modified by the operation
    pub fn paint_tiles(
        &mut self,
        id: LayerId,
        coords: impl IntoIterator<Item = HexCoord>,
    ) -> Vec<HexCoord> {
        let result = match self
            .inner
            .iter_mut()
            .find(|layer| layer.id == id)
            .map(|layer| &mut layer.kind)
        {
            Some(LayerInner::Tiles(t)) => t.paint_multiple(coords),
            _ => Vec::new(),
        };
        if !result.is_empty() {
            self.change_revision();
        }
        result
    }

    /// Erases multiple tiles in once pass on a given layer
    /// Returns true if the operation caused a change.
    pub fn erase_tiles(
        &mut self,
        id: LayerId,
        coords: impl IntoIterator<Item = HexCoord>,
    ) -> Vec<HexCoord> {
        let result = match self
            .inner
            .iter_mut()
            .find(|layer| layer.id == id)
            .map(|layer| &mut layer.kind)
        {
            Some(LayerInner::Tiles(t)) => t.erase_multiple(coords),
            _ => Vec::new(),
        };
        if !result.is_empty() {
            self.change_revision();
        }
        result
    }
}

impl Default for Scene {
    fn default() -> Self {
        let kind = LayerInner::Tiles(SparseTiles::new(DEFAULT_COLORS[0]));
        let layer = Layer::new("Layer 1", kind);

        Self {
            inner: vec![layer],
            revision: 0,
            assets: AssetStore::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiles_layer(name: &str) -> Layer {
        Layer::new(name, LayerInner::Tiles(SparseTiles::new(DEFAULT_COLORS[0])))
    }

    #[test]
    fn default_scene_has_a_single_visible_layer() {
        let scene = Scene::default();
        assert_eq!(scene.inner.len(), 1);
        assert_eq!(scene.inner[0].name, "Layer 1");
        assert!(scene.inner[0].visible);
        assert_eq!(scene.revision(), 0);
    }

    #[test]
    fn new_kind_cycles_through_default_colours_by_layer_count() {
        let mut scene = Scene::default(); // starts with 1 layer
        let LayerInner::Tiles(tiles) = scene.new_kind(LayerKind::Tiles) else {
            panic!("expected a Tiles layer");
        };
        assert_eq!(tiles.colour, DEFAULT_COLORS[1]);

        // Push layers until the palette wraps around.
        for _ in 0..4 {
            let layer = Layer::new("filler", scene.new_kind(LayerKind::Tiles));
            let len = scene.inner.len();
            scene.insert_layer(layer, len);
        }
        assert_eq!(scene.inner.len(), 5);
        let LayerInner::Tiles(tiles) = scene.new_kind(LayerKind::Tiles) else {
            panic!("expected a Tiles layer");
        };
        assert_eq!(tiles.colour, DEFAULT_COLORS[0], "palette should wrap");
    }

    #[test]
    fn new_kind_of_noise_and_image() {
        let scene = Scene::default();
        assert!(matches!(
            scene.new_kind(LayerKind::Noise),
            LayerInner::Perlin(_)
        ));
        assert!(matches!(
            scene.new_kind(LayerKind::Image),
            LayerInner::Image(_)
        ));
    }

    #[test]
    fn insert_and_get_layer_round_trip() {
        let mut scene = Scene::default();
        let layer = tiles_layer("New");
        let id = layer.id;

        scene.insert_layer(layer, 1);

        assert_eq!(scene.inner.len(), 2);
        assert_eq!(scene.get_layer(id).unwrap().name, "New");
    }

    #[test]
    fn insert_layer_bumps_the_revision() {
        let mut scene = Scene::default();
        let before = scene.revision();
        scene.insert_layer(tiles_layer("New"), 0);
        assert_ne!(scene.revision(), before);
    }

    #[test]
    fn remove_layer_returns_the_layer_and_its_index() {
        let mut scene = Scene::default();
        let layer = tiles_layer("New");
        let id = layer.id;
        scene.insert_layer(layer, 1);

        let (removed, index) = scene.remove_layer(id).unwrap();
        assert_eq!(removed.id, id);
        assert_eq!(index, 1);
        assert_eq!(scene.inner.len(), 1);
        assert!(scene.get_layer(id).is_none());
    }

    #[test]
    fn remove_layer_of_unknown_id_is_none() {
        let mut scene = Scene::default();
        assert!(scene.remove_layer(LayerId::from_raw(u64::MAX)).is_none());
    }

    #[test]
    fn move_layer_swaps_the_two_positions() {
        // Documents current behaviour: `move_layer` swaps the two slots
        // rather than shifting everything in between.
        let mut scene = Scene::default();
        let a = scene.inner[0].id; // index 0
        let b = tiles_layer("B");
        let b_id = b.id;
        scene.insert_layer(b, 1); // index 1
        let c = tiles_layer("C");
        let c_id = c.id;
        scene.insert_layer(c, 2); // index 2

        let from = scene.move_layer(a, 2).unwrap();
        assert_eq!(from, 0);
        assert_eq!(scene.inner[0].id, c_id);
        assert_eq!(scene.inner[1].id, b_id);
        assert_eq!(scene.inner[2].id, a);
    }

    #[test]
    fn get_layer_mut_allows_mutation_and_bumps_revision() {
        let mut scene = Scene::default();
        let id = scene.inner[0].id;
        let before = scene.revision();

        scene.get_layer_mut(id).unwrap().name = "Renamed".into();

        assert_eq!(scene.get_layer(id).unwrap().name, "Renamed");
        assert_ne!(scene.revision(), before);
    }

    #[test]
    fn paint_and_erase_tile_report_whether_they_changed_anything() {
        let mut scene = Scene::default();
        let id = scene.inner[0].id;
        let coord = HexCoord { col: 0, row: 0 };

        assert!(
            scene.paint_tile(id, coord),
            "first paint should change something"
        );
        assert!(
            !scene.paint_tile(id, coord),
            "painting an already-painted tile is a no-op"
        );

        assert!(
            scene.erase_tile(id, coord),
            "first erase should change something"
        );
        assert!(
            !scene.erase_tile(id, coord),
            "erasing an already-empty tile is a no-op"
        );
    }

    #[test]
    fn paint_tile_on_missing_or_wrong_kind_layer_is_false() {
        let mut scene = Scene::default();
        assert!(!scene.paint_tile(LayerId::from_raw(u64::MAX), HexCoord { col: 0, row: 0 }));

        let noise_layer = Layer::new("Noise", scene.new_kind(LayerKind::Noise));
        let noise_id = noise_layer.id;
        scene.insert_layer(noise_layer, 1);
        assert!(!scene.paint_tile(noise_id, HexCoord { col: 0, row: 0 }));
    }

    #[test]
    fn paint_tiles_returns_only_the_tiles_actually_changed() {
        let mut scene = Scene::default();
        let id = scene.inner[0].id;
        let a = HexCoord { col: 0, row: 0 };
        let b = HexCoord { col: 1, row: 0 };

        scene.paint_tile(id, a); // pre-paint `a`

        let changed = scene.paint_tiles(id, vec![a, b]);
        assert_eq!(changed, vec![b], "already-painted `a` should be excluded");
    }

    #[test]
    fn erase_tiles_returns_only_the_tiles_actually_changed() {
        let mut scene = Scene::default();
        let id = scene.inner[0].id;
        let a = HexCoord { col: 0, row: 0 };
        let b = HexCoord { col: 1, row: 0 };
        scene.paint_tiles(id, vec![a, b]);

        let changed = scene.erase_tiles(id, vec![a, HexCoord { col: 9, row: 9 }]);
        assert_eq!(
            changed,
            vec![a],
            "the never-painted tile should be excluded"
        );
    }

    #[test]
    fn get_visible_layers_filters_out_hidden_layers() {
        let mut scene = Scene::default(); // 1 visible layer
        let hidden = tiles_layer("Hidden");
        let hidden_id = hidden.id;
        scene.insert_layer(hidden, 1);
        scene.get_layer_mut(hidden_id).unwrap().visible = false;

        assert_eq!(scene.inner.len(), 2);
        assert_eq!(scene.get_visible_layers().len(), 1);
    }
}
