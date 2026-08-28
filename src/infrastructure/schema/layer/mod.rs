mod image;
mod noise;
mod tiles;

pub use image::{ImageLayerV1, RectangleV1};
pub use noise::PerlinV1;
pub use tiles::{ColourV1, HexCoordV1, TilesV1};

use serde::{Deserialize, Serialize};

use crate::domain::{
    Layer, LayerInner,
    id::{ImageId, LayerId},
    layer::unknown::UnknownLayer,
};

/// One entry in `scene.json`'s ordered layer list.
/// Stores the the layers place in the stack & where to find its data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerEntryV1 {
    pub id: u64,
    pub name: String,
    pub visible: bool,
    pub kind: String,
    pub file: String,
}

impl From<&Layer> for LayerEntryV1 {
    fn from(layer: &Layer) -> Self {
        let id = layer.id.raw();
        let kind = match &layer.kind {
            LayerInner::Tiles(store) if store.is_inverted() => "InvertedTiles".to_string(),
            LayerInner::Tiles(_) => "Tiles".to_string(),
            LayerInner::Perlin(_) => "Perlin".to_string(),
            LayerInner::Image(_) => "Image".to_string(),
            LayerInner::Unknown(unknown) => unknown.kind.clone(),
        };

        LayerEntryV1 {
            id,
            name: layer.name.clone(),
            visible: layer.visible,
            kind,
            file: format!("layers/{id}.json"),
        }
    }
}

/// Known payload of `layers/<id>.json`. Tagged by `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LayerDataV1 {
    Tiles(TilesV1),
    InvertedTiles(TilesV1),
    Perlin(PerlinV1),
    Image(ImageLayerV1),
}

impl LayerDataV1 {
    fn from_known(inner: &LayerInner) -> Option<Self> {
        Some(match inner {
            LayerInner::Tiles(store) if store.is_inverted() => {
                LayerDataV1::InvertedTiles(store.into())
            }
            LayerInner::Tiles(store) => LayerDataV1::Tiles(store.into()),
            LayerInner::Perlin(perlin) => LayerDataV1::Perlin(perlin.into()),
            LayerInner::Image(image) => LayerDataV1::Image(image.into()),
            LayerInner::Unknown(_) => return None,
        })
    }

    fn into_domain(self, resolve_image: impl Fn(u64) -> Option<ImageId>) -> LayerInner {
        match self {
            LayerDataV1::Tiles(tiles) => LayerInner::Tiles(tiles.into_domain(false)),
            LayerDataV1::InvertedTiles(tiles) => LayerInner::Tiles(tiles.into_domain(true)),
            LayerDataV1::Perlin(perlin) => LayerInner::Perlin(perlin.into()),
            LayerDataV1::Image(image) => LayerInner::Image(image.into_domain(resolve_image)),
        }
    }
}

/// The full contents of `layers/<id>.json`.
/// Handles known layer data formats and unrecognized ones.
#[derive(Debug, Clone)]
pub enum LayerPayload {
    Known(LayerDataV1),
    Unknown { kind: String, raw: Vec<u8> },
}

impl LayerPayload {
    /// `kind` comes from the layer's `LayerEntryV1`, used as a fallback tag
    /// if `bytes` don't parse as any known `LayerDataV1` variant.
    pub fn parse(kind: &str, bytes: &[u8]) -> Self {
        match serde_json::from_slice(bytes) {
            Ok(data) => LayerPayload::Known(data),
            Err(_) => LayerPayload::Unknown {
                kind: kind.to_string(),
                raw: bytes.to_vec(),
            },
        }
    }

    pub fn to_bytes(&self) -> serde_json::Result<Vec<u8>> {
        match self {
            LayerPayload::Known(data) => serde_json::to_vec_pretty(data),
            LayerPayload::Unknown { raw, .. } => Ok(raw.clone()),
        }
    }
}

impl From<&LayerInner> for LayerPayload {
    fn from(inner: &LayerInner) -> Self {
        match inner {
            LayerInner::Unknown(unknown) => LayerPayload::Unknown {
                kind: unknown.kind.clone(),
                raw: unknown.raw.clone(),
            },
            known => LayerPayload::Known(
                LayerDataV1::from_known(known)
                    .expect("every non-Unknown LayerInner variant converts"),
            ),
        }
    }
}

/// Assembles the `(manifest entry, payload)` pair for one layer.
pub fn layer_to_wire(layer: &Layer) -> (LayerEntryV1, LayerPayload) {
    (LayerEntryV1::from(layer), LayerPayload::from(&layer.kind))
}

/// Reconstructs a domain [`Layer`] from its manifest entry and payload.
/// `resolve_image` maps a persisted resource id to the `ImageId` it was
/// registered under this session - see `schema::Document::into_scene`.
pub fn layer_from_wire(
    entry: LayerEntryV1,
    payload: LayerPayload,
    resolve_image: impl Fn(u64) -> Option<ImageId>,
) -> Layer {
    let kind = match payload {
        LayerPayload::Known(data) => data.into_domain(resolve_image),
        LayerPayload::Unknown { kind, raw } => LayerInner::Unknown(UnknownLayer { kind, raw }),
    };

    Layer {
        id: LayerId::from_raw(entry.id),
        name: entry.name,
        visible: entry.visible,
        kind,
    }
}
