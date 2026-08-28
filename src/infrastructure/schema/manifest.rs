use serde::{Deserialize, Serialize};

use super::layer::LayerEntryV1;
use super::{CURRENT_VERSION, FORMAT_ID};

pub const DEFAULT_SCENE_FILE: &str = "scene.json";
pub const DEFAULT_RESOURCES_FILE: &str = "resources.json";

/// `metadata.json`
///
/// Shape should rarely change.
/// Content is reached by following the contained file pointers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataV1 {
    pub format: String,
    pub version: u32,
    pub scene_file: String,
    pub resources_file: String,

    /// Humane facing metadata.
    /// Loosely bound as more fields are bound to be added in the future.
    #[serde(default)]
    pub name: Option<String>,
}

impl MetadataV1 {
    pub fn new(name: Option<String>) -> Self {
        MetadataV1 {
            format: FORMAT_ID.to_string(),
            version: CURRENT_VERSION,
            scene_file: DEFAULT_SCENE_FILE.to_string(),
            resources_file: DEFAULT_RESOURCES_FILE.to_string(),
            name,
        }
    }
}

/// `scene.json`
///
/// Ordered list of layers, pointing to their individual content.
/// Kept seperate from metadata so anything that needs to tell
/// two documents apart can check just `metadata.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneManifestV1 {
    pub active_layer: Option<u64>,
    pub layers: Vec<LayerEntryV1>,
}
