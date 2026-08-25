use std::collections::HashSet;

use iced::Color;
use rand::random;

use crate::domain::{
    HexCoord, Layer, LayerInner, LayerKind,
    assets::AssetStore,
    id::LayerId,
    layer::{LayerInnerImpl, noise::PerlinNoiseLayer, tiles::SparseTiles},
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
    pub fn from_layers(inner: Vec<Layer>) -> Self {
        Self {
            inner,
            revision: 1,
            assets: AssetStore::default(),
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
    pub fn paint_tiles(&mut self, id: LayerId, coords: HashSet<HexCoord>) -> HashSet<HexCoord> {
        let result = match self
            .inner
            .iter_mut()
            .find(|layer| layer.id == id)
            .map(|layer| &mut layer.kind)
        {
            Some(LayerInner::Tiles(t)) => t.paint_multiple(coords),
            _ => HashSet::new(),
        };
        if !result.is_empty() {
            self.change_revision();
        }
        result
    }

    /// Erases multiple tiles in once pass on a given layer
    /// Returns true if the operation caused a change.
    pub fn erase_tiles(&mut self, id: LayerId, coords: HashSet<HexCoord>) -> HashSet<HexCoord> {
        let result = match self
            .inner
            .iter_mut()
            .find(|layer| layer.id == id)
            .map(|layer| &mut layer.kind)
        {
            Some(LayerInner::Tiles(t)) => t.erase_multiple(coords),
            _ => HashSet::new(),
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
