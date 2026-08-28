use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::assets::AssetStore;

/// An entry in `resources.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEntryV1 {
    pub id: u64,
    pub file: String,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceManifestV1 {
    pub entries: Vec<ResourceEntryV1>,
}

impl ResourceManifestV1 {
    /// Builds the manifest entries for every image asset in `assets`,
    /// alongside the source bytes to write into `resources/`.
    pub fn from_assets(assets: &AssetStore) -> (Self, BTreeMap<u64, Vec<u8>>) {
        let mut entries = Vec::new();
        let mut bytes = BTreeMap::new();

        for (id, name, extension, encoded) in assets.iter_images() {
            let raw_id = id.raw();
            entries.push(ResourceEntryV1 {
                id: raw_id,
                file: format!("resources/{raw_id}.{extension}"),
                name: name.to_string(),
                kind: "image".to_string(),
            });
            bytes.insert(raw_id, encoded.to_vec());
        }

        (ResourceManifestV1 { entries }, bytes)
    }
}
