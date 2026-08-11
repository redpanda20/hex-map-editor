//! ## Adding a new version
//! When the live state gains a field/variant that should be persisted:
//! 1. Add a new `DocumentV{N}` (and any nested types it needs) below,
//!    normally by copying the previous version and changing what's needed.
//! 2. Bump `CURRENT_VERSION` and change the `Document` alias to point at
//!    the new type.
//! 3. Add a `From<DocumentV{N-1}> for DocumentV{N}` migration and a new
//!    match arm in `migrate_to_latest`.
//! 4. Update the conversions in `convert.rs` for the new fields.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Bump whenever the `Envelope` changes
const FORMAT_ID: &str = "hex-map-editor";

/// Bump whenever `Document` changes
pub const CURRENT_VERSION: u32 = 1;
pub type SceneV1 = DocumentV1;

/// The envelope written to disk
///
/// `data` is kept as an untyped [`Value`] so that the concrete version of `data` can be chosen *after* looking at
/// `version`, which is what makes forward migration possible.
#[derive(Debug, Serialize, Deserialize)]
struct Envelope {
    format: String,
    version: u32,
    data: Value,
}

#[derive(Debug)]
pub enum LoadError {
    /// The file isn't a hex-map-editor save file at all.
    NotAProjectFile,
    /// The file claims a version newer than this build understands.
    UnsupportedVersion(u32),
    /// The envelope or payload failed to parse as JSON.
    Malformed(serde_json::Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::NotAProjectFile => {
                write!(f, "This file doesn't look like a hex-map-editor project.")
            }
            LoadError::UnsupportedVersion(v) => write!(
                f,
                "This project was saved with a newer version of the editor (format v{v}). Please update the app."
            ),
            LoadError::Malformed(err) => write!(f, "Save file is corrupted: {err}"),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<serde_json::Error> for LoadError {
    fn from(err: serde_json::Error) -> Self {
        LoadError::Malformed(err)
    }
}

/// Serializes the given document into the on-disk envelope format.
pub fn serialize(document: &SceneV1) -> serde_json::Result<Vec<u8>> {
    let envelope = Envelope {
        format: FORMAT_ID.to_string(),
        version: CURRENT_VERSION,
        data: serde_json::to_value(document)?,
    };
    serde_json::to_vec_pretty(&envelope)
}

/// Parses bytes from disk into the latest [`Document`]
pub fn deserialize(bytes: &[u8]) -> Result<SceneV1, LoadError> {
    let envelope: Envelope =
        serde_json::from_slice(bytes).map_err(|_| LoadError::NotAProjectFile)?;

    if envelope.format != FORMAT_ID {
        return Err(LoadError::NotAProjectFile);
    }

    migrate_to_latest(envelope.version, envelope.data)
}

/// Migrates a payload of the given `version` up to [`Document`] (`CURRENT_VERSION`).
fn migrate_to_latest(version: u32, data: Value) -> Result<SceneV1, LoadError> {
    match version {
        1 => Ok(serde_json::from_value::<DocumentV1>(data)?),

        // Example of what a future migration looks like once DocumentV2 exists:
        //
        // 2 => Ok(serde_json::from_value::<DocumentV2>(data)?),
        // 1 => {
        //     let v1 = serde_json::from_value::<DocumentV1>(data)?;
        //     Ok(DocumentV2::from(v1))
        // }
        v => Err(LoadError::UnsupportedVersion(v)),
    }
}

// ---------------------------------------------------------------------------
// v1 schema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentV1 {
    pub layers: Vec<LayerV1>,
    pub active_layer: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerV1 {
    pub name: String,
    pub visible: bool,
    pub kind: LayerKindV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LayerKindV1 {
    Tiles {
        colour: ColourV1,
        tiles: Vec<HexCoordV1>,
    },
    InvertedTiles {
        colour: ColourV1,
        tiles: Vec<HexCoordV1>,
    },
    Perlin {
        seed: u64,
        threshold: f32,
        scale: f32,
        octaves: NoiseOctavesV1,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NoiseOctavesV1 {
    One,
    Many { count: usize, persistence: f32 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HexCoordV1 {
    pub col: i32,
    pub row: i32,
}

/// Colour stored as plain floats rather than depending on `iced::Color`
/// directly, so the save format doesn't break if iced changes its type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColourV1 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
