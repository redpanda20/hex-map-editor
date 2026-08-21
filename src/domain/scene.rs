use iced::Color;

use crate::domain::{
    HexCoord, Layer, Tool, flood_fill,
    history::Edit,
    layer_inner::LayerKind,
    layer_inner::{LayerInner, NoiseOctaves, PerlinNoiseLayer, SparseTiles},
};

const DEFAULT_COLORS: [Color; 5] = [
    Color::from_rgba8(245, 196, 168, 0.9),
    Color::from_rgba8(168, 212, 176, 0.9),
    Color::from_rgba8(168, 200, 245, 0.9),
    Color::from_rgba8(196, 168, 245, 0.9),
    Color::from_rgba8(245, 168, 200, 0.9),
];

#[derive(Debug, Clone)]
pub enum SceneMessage {
    AddLayer(String, LayerKind),
    RemoveLayer(usize),
    SwapLayers(usize, usize),

    EditLayerVisibility(usize, bool),
    EditLayerName(usize, String),
    EditLayerFistColour(usize, Color),
    EditLayerSeed(usize, u64),
    EditLayerScale(usize, f32),
    EditLayerThreshold(usize, f32),
    EditLayerOctaves(usize, usize),
    EditLayerPersistence(usize, f32),

    PaintHex(HexCoord),
    EraseHex(HexCoord),
    FillFromHex(HexCoord),

    ChangeTool(Tool),
    SetActiveLayer(Option<usize>),
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub inner: Vec<Layer>,
    pub active_layer: Option<usize>,
    pub tool: Tool,

    revision: u64,
}

impl Scene {
    pub fn from_layers(inner: Vec<Layer>) -> Self {
        Self {
            inner,
            active_layer: None,
            tool: Tool::default(),
            revision: 1,
        }
    }

    /// TODO: Check which names are in use and pick the smallest number
    fn canonacalize_name(&self, name: String) -> String {
        let layer_count = self.inner.len() + 1;
        format!("{name} {layer_count}")
    }

    /// A counter incremented when the state of Layers changes.
    /// Used by consumers to invalidate out of date caches
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Applies `SceneMessage`.
    ///
    /// If content changed, the relevant `Edit` is returned.
    /// None if message didn't change content, or if it was a no-op
    pub fn update(&mut self, message: SceneMessage) -> Option<Edit> {
        // Apply non-revertable changes
        match message {
            SceneMessage::ChangeTool(tool) => self.tool = tool,
            SceneMessage::SetActiveLayer(active_layer) => self.active_layer = active_layer,
            _ => {}
        };

        let edit = self.command_to_edit(message);

        if let Some(edit) = &edit {
            self.apply_edit(edit, true);
        }

        edit
    }

    /// Reverses a single `Edit` (sets the `before` value). Used by `History::undo`.
    pub fn undo_edit(&mut self, edit: &Edit) {
        self.revision = self.revision.wrapping_add(1);
        self.apply_edit(edit, false);
    }

    /// Re-applies a single `Edit` (sets the `after` value). Used by `History::redo`.
    pub fn redo_edit(&mut self, edit: &Edit) {
        self.revision = self.revision.wrapping_add(1);
        self.apply_edit(edit, true);
    }

    fn command_to_edit(&self, message: SceneMessage) -> Option<Edit> {
        let edit = match message {
            // --- Edit tools ---
            SceneMessage::ChangeTool(_) => {
                return None;
            }
            SceneMessage::SetActiveLayer(_) => {
                return None;
            }

            // --- Edit Layers ---
            SceneMessage::AddLayer(name, layer_type) => {
                let name = self.canonacalize_name(name);
                let inner = match layer_type {
                    LayerKind::Tiles => {
                        let colour = DEFAULT_COLORS[self.inner.len() % 5];
                        LayerInner::Tiles(SparseTiles::new(colour))
                    }
                    LayerKind::PerlinNoise => {
                        let seed = rand::random();
                        LayerInner::Perlin(PerlinNoiseLayer::new(seed))
                    }
                };
                let layer = Layer {
                    name,
                    visible: true,
                    inner,
                };
                let index = self.inner.len();

                Edit::LayerAdded { index, layer }
            }

            SceneMessage::RemoveLayer(index) => {
                let layer = self.inner.get(index)?.clone();
                Edit::LayerRemoved { index, layer }
            }
            SceneMessage::SwapLayers(a, b) => {
                if a >= self.inner.len() || b >= self.inner.len() || a == b {
                    return None;
                }
                Edit::LayersSwapped { a, b }
            }

            // --- Edit layer properties ---
            SceneMessage::EditLayerVisibility(index, new_visibility) => {
                let before = self.inner.get(index)?.visible;
                if before == new_visibility {
                    return None;
                }
                Edit::LayerVisibility {
                    index,
                    before,
                    after: new_visibility,
                }
            }
            SceneMessage::EditLayerName(index, new_name) => {
                let layer = self.inner.get(index)?;
                if layer.name == new_name {
                    return None;
                }
                Edit::LayerName {
                    index,
                    before: layer.name.clone(),
                    after: new_name,
                }
            }
            SceneMessage::EditLayerFistColour(index, new_colour) => {
                let before = match &self.inner.get(index)?.inner {
                    LayerInner::Tiles(store) | LayerInner::InvertedTiles(store) => {
                        store.get_colour()
                    }
                    LayerInner::Perlin(_) => todo!(),
                };
                if before == new_colour {
                    return None;
                }
                Edit::LayerColour {
                    index,
                    before,
                    after: new_colour,
                }
            }
            // --- Edit Layer Properties (Proc gen) ---
            SceneMessage::EditLayerSeed(index, seed) => {
                let LayerInner::Perlin(perlin) = &self.inner.get(index)?.inner else {
                    return None;
                };
                let before = perlin.seed;
                if before == seed {
                    return None;
                }
                Edit::LayerSeed {
                    index,
                    before,
                    after: seed,
                }
            }
            SceneMessage::EditLayerScale(index, scale) => {
                let LayerInner::Perlin(perlin) = &self.inner.get(index)?.inner else {
                    return None;
                };
                let before = perlin.frequency;
                if before == scale {
                    return None;
                }
                Edit::LayerScale {
                    index,
                    before,
                    after: scale,
                }
            }
            SceneMessage::EditLayerThreshold(index, threshold) => {
                let LayerInner::Perlin(perlin) = &self.inner.get(index)?.inner else {
                    return None;
                };
                let before = perlin.threshold;
                if before == threshold {
                    return None;
                }
                Edit::LayerThreshold {
                    index,
                    before,
                    after: threshold,
                }
            }
            SceneMessage::EditLayerOctaves(index, count) => {
                let LayerInner::Perlin(perlin) = &self.inner.get(index)?.inner else {
                    return None;
                };
                let before = perlin.octaves;
                let current_persistence = match before {
                    NoiseOctaves::One => None,
                    NoiseOctaves::Many { persistence, .. } => Some(persistence),
                };
                let after = if count == 1 {
                    NoiseOctaves::One
                } else {
                    NoiseOctaves::Many {
                        count,
                        persistence: current_persistence.unwrap_or(0.5),
                    }
                };
                if before == after {
                    return None;
                }
                Edit::LayerOctaves {
                    index,
                    before,
                    after,
                }
            }
            SceneMessage::EditLayerPersistence(index, new_persistence) => {
                let LayerInner::Perlin(perlin) = &self.inner.get(index)?.inner else {
                    return None;
                };
                let NoiseOctaves::Many {
                    persistence: before,
                    ..
                } = perlin.octaves
                else {
                    return None;
                };
                if before == new_persistence {
                    return None;
                }
                Edit::LayerPersistence {
                    index,
                    before,
                    after: new_persistence,
                }
            }

            // --- Edit tiles ---
            SceneMessage::PaintHex(hex_coord) => {
                let active_index = self.active_layer?;
                let layer = self.inner.get(active_index)?;

                let (before, after) = match &layer.inner {
                    LayerInner::Tiles(store) => (store.tiles.contains(&hex_coord), true),
                    LayerInner::InvertedTiles(store) => (!store.tiles.contains(&hex_coord), false),
                    // Noise layer cannot be drawn to
                    LayerInner::Perlin(_) => return None,
                };

                if before == after {
                    return None;
                }

                Edit::Tile {
                    layer: active_index,
                    coord: hex_coord,
                    before,
                    after,
                }
            }

            SceneMessage::EraseHex(hex_coord) => {
                let active_index = self.active_layer?;
                let layer = self.inner.get(active_index)?;

                let (before, after) = match &layer.inner {
                    LayerInner::Tiles(store) => (store.tiles.contains(&hex_coord), false),
                    LayerInner::InvertedTiles(store) => (!store.tiles.contains(&hex_coord), true),
                    // Noise layer cannot be drawn to
                    LayerInner::Perlin(_) => return None,
                };

                if before == after {
                    return None;
                }

                Edit::Tile {
                    layer: active_index,
                    coord: hex_coord,
                    before,
                    after,
                }
            }

            SceneMessage::FillFromHex(hex_coord) => {
                let index = self.active_layer?;
                let layer = self.inner.get(index)?;
                let store = match &layer.inner {
                    LayerInner::Tiles(store) | LayerInner::InvertedTiles(store) => store,
                    LayerInner::Perlin(_) => return None,
                };
                // Tiles layers fill by painting the region; InvertedTiles fill by erasing it.
                let insert = matches!(layer.inner, LayerInner::Tiles(_));

                // If the layer is empty, short circuit and invert
                let Some(bounds) = layer.inner.get_bounds() else {
                    return Some(Edit::LayerInverted { index });
                };

                // If the layer has no bound, short circuit and invert
                let Some(region) = flood_fill(hex_coord, store.get_all_tiles(), bounds) else {
                    return Some(Edit::LayerInverted { index });
                };

                let edits = region
                    .into_iter()
                    .filter_map(|coord| {
                        let before = layer.inner.exists_at(&coord);
                        (before != insert).then_some(Edit::Tile {
                            layer: index,
                            coord,
                            before,
                            after: insert,
                        })
                    })
                    .collect::<Vec<_>>();
                Edit::coalesce(edits)?
            }
        };

        Some(edit)
    }

    /// The single place which mutates state of `Scene`.
    /// Can perform changes in forward or reverse
    ///
    /// `edit` is applied to Scene's state
    /// `forward` selects if the change is done or undone
    fn apply_edit(&mut self, edit: &Edit, forward: bool) {
        self.revision = self.revision.wrapping_add(1);

        match edit {
            Edit::Batch { edits } => {
                if forward {
                    for edit in edits {
                        self.apply_edit(edit, true);
                    }
                } else {
                    for edit in edits.iter().rev() {
                        self.apply_edit(edit, false);
                    }
                }
            }

            Edit::Tile {
                layer,
                coord,
                before,
                after,
            } => {
                if let Some(store) = self.inner.get_mut(*layer).and_then(|l| match &mut l.inner {
                    LayerInner::Tiles(store) | LayerInner::InvertedTiles(store) => Some(store),
                    LayerInner::Perlin(_) => None,
                }) {
                    let is_tile_present = match forward {
                        true => *after,
                        false => *before,
                    };
                    if is_tile_present {
                        store.tiles.insert(*coord);
                    } else {
                        store.tiles.remove(coord);
                    }
                }
            }

            Edit::LayerAdded { index, layer } => {
                if forward {
                    self.inner.insert(*index, layer.clone());
                } else if *index < self.inner.len() {
                    self.inner.remove(*index);
                }
            }
            Edit::LayerRemoved { index, layer } => {
                if forward {
                    if *index < self.inner.len() {
                        self.inner.remove(*index);
                    }
                } else {
                    self.inner.insert(*index, layer.clone());
                }
            }
            Edit::LayersSwapped { a, b } => {
                if *a < self.inner.len() && *b < self.inner.len() {
                    self.inner.swap(*a, *b);
                }
            }
            Edit::LayerInverted { index } => {
                if let Some(layer) = self.inner.get_mut(*index) {
                    layer.inner = match &layer.inner {
                        LayerInner::Tiles(store) => LayerInner::InvertedTiles(store.clone()),
                        LayerInner::InvertedTiles(store) => LayerInner::Tiles(store.clone()),
                        LayerInner::Perlin(_) => return,
                    };
                }
            }

            Edit::LayerVisibility {
                index,
                before,
                after,
            } => {
                if let Some(layer) = self.inner.get_mut(*index) {
                    layer.visible = if forward { *after } else { *before };
                }
            }
            Edit::LayerName {
                index,
                before,
                after,
            } => {
                if let Some(layer) = self.inner.get_mut(*index) {
                    layer.name = if forward {
                        after.clone()
                    } else {
                        before.clone()
                    };
                }
            }
            Edit::LayerColour {
                index,
                before,
                after,
            } => {
                if let Some(layer) = self.inner.get_mut(*index) {
                    let colour = if forward { *after } else { *before };
                    match &mut layer.inner {
                        LayerInner::Tiles(store) | LayerInner::InvertedTiles(store) => {
                            store.set_colour(colour)
                        }
                        LayerInner::Perlin(_) => (),
                    }
                }
            }

            Edit::LayerSeed {
                index,
                before,
                after,
            } => {
                if let Some(Layer {
                    inner: LayerInner::Perlin(perlin),
                    ..
                }) = self.inner.get_mut(*index)
                {
                    perlin.set_seed(if forward { *after } else { *before });
                }
            }
            Edit::LayerScale {
                index,
                before,
                after,
            } => {
                if let Some(Layer {
                    inner: LayerInner::Perlin(perlin),
                    ..
                }) = self.inner.get_mut(*index)
                {
                    perlin.set_frequency(if forward { *after } else { *before });
                }
            }
            Edit::LayerThreshold {
                index,
                before,
                after,
            } => {
                if let Some(Layer {
                    inner: LayerInner::Perlin(perlin),
                    ..
                }) = self.inner.get_mut(*index)
                {
                    perlin.set_threshold(if forward { *after } else { *before });
                }
            }
            Edit::LayerOctaves {
                index,
                before,
                after,
            } => {
                if let Some(Layer {
                    inner: LayerInner::Perlin(perlin),
                    ..
                }) = self.inner.get_mut(*index)
                {
                    perlin.octaves = if forward { *after } else { *before };
                }
            }
            Edit::LayerPersistence {
                index,
                before,
                after,
            } => {
                if let Some(Layer {
                    inner:
                        LayerInner::Perlin(PerlinNoiseLayer {
                            octaves: NoiseOctaves::Many { persistence, .. },
                            ..
                        }),
                    ..
                }) = self.inner.get_mut(*index)
                {
                    *persistence = if forward { *after } else { *before };
                }
            }
        }
    }

    pub fn get_visible_layers(&self) -> Vec<&LayerInner> {
        self.inner
            .iter()
            .filter(|layer| layer.visible)
            .map(|layer| &layer.inner)
            .collect::<Vec<_>>()
    }
}

impl Default for Scene {
    fn default() -> Self {
        let inner = LayerInner::Tiles(SparseTiles::new(DEFAULT_COLORS[0]));

        let layer = Layer {
            name: "Layer 1".to_string(),
            visible: true,
            inner,
        };

        Self {
            inner: vec![layer],
            active_layer: Some(0),
            tool: Tool::default(),
            revision: 0,
        }
    }
}
