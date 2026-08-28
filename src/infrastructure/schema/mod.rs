//! On-disk save format for hex-map-editor.
//!
//! ## Layout inside the archive
//! ```text
//! map.hexmap (zip)
//! ├── metadata.json        metadata + pointers to the files below
//! ├── scene.json           ordered layer manifest + per-layer file pointers
//! ├── layers/
//! │    ├── 1.json          one file per layer, tagged by kind
//! │    └── 2.json
//! ├── resources.json       resource manifest: id -> {file, name, kind}
//! └── resources/
//!      ├── 3.png           binary assets, referenced by id from layers
//!      └── 4.webp
//! ```

pub mod layer;
pub mod manifest;
pub mod resource;

use std::collections::BTreeMap;

use crate::domain::{
    Scene,
    assets::AssetStore,
    id::{ImageId, LayerId},
};
use crate::infrastructure::archive::{self, ArchiveError};
use crate::infrastructure::image_codec::decode_image_asset;

pub use layer::{LayerDataV1, LayerEntryV1, LayerPayload};
pub use manifest::{MetadataV1, SceneManifestV1};
pub use resource::{ResourceEntryV1, ResourceManifestV1};

/// Bump whenever `MetadataV1` changes shape.
const FORMAT_ID: &str = "hex-map-editor";

/// Bump whenever the archive layout or any manifest/layer/resource type
/// changes in a way `serde`'s defaulting can't absorb.
pub const CURRENT_VERSION: u32 = 2;

pub const METADATA_FILE: &str = "metadata.json";

#[derive(Debug)]
pub enum LoadError {
    /// The file isn't a hex-map-editor project at all.
    NotAProjectFile,
    /// The file claims a version newer than this build understands.
    UnsupportedVersion(u32),
    /// A manifest pointed at a file that isn't in the archive.
    MissingEntry(String),
    /// A manifest or referenced file failed to parse.
    Malformed(String),
    Archive(ArchiveError),
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
            LoadError::MissingEntry(path) => write!(f, "Save file is corrupted: missing {path}"),
            LoadError::Malformed(err) => write!(f, "Save file is corrupted: {err}"),
            LoadError::Archive(err) => write!(f, "Save file is corrupted: {err}"),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<ArchiveError> for LoadError {
    fn from(err: ArchiveError) -> Self {
        LoadError::Archive(err)
    }
}

impl From<serde_json::Error> for LoadError {
    fn from(err: serde_json::Error) -> Self {
        LoadError::Malformed(err.to_string())
    }
}

/// The parsed, in-memory contents of a `.hexmap` archive.
#[derive(Debug, Clone)]
pub struct Document {
    pub metadata: MetadataV1,
    pub scene: SceneManifestV1,
    /// Layer id -> payload, for every layer listed in `scene.layers`.
    pub layers: BTreeMap<u64, LayerPayload>,
    pub resources: ResourceManifestV1,
    /// Resource id -> raw bytes, for every resource listed in `resources`.
    pub resource_bytes: BTreeMap<u64, Vec<u8>>,
}

impl Document {
    /// Assembles a `Document` from the live scene, ready to serialize.
    pub fn from_scene(scene: &Scene, name: Option<String>) -> Self {
        let mut layers = BTreeMap::new();
        let mut scene_layers = Vec::with_capacity(scene.inner.len());

        for domain_layer in &scene.inner {
            let (entry, payload) = layer::layer_to_wire(domain_layer);
            layers.insert(entry.id, payload);
            scene_layers.push(entry);
        }

        let (resources, resource_bytes) = ResourceManifestV1::from_assets(&scene.assets);

        Document {
            metadata: MetadataV1::new(name),
            scene: SceneManifestV1 {
                active_layer: None,
                layers: scene_layers,
            },
            layers,
            resources,
            resource_bytes,
        }
    }

    /// Reconstructs a live [`Scene`] from a parsed `Document`.
    pub fn into_scene(self) -> Scene {
        let mut assets = AssetStore::default();
        let mut image_ids = BTreeMap::new();

        for entry in &self.resources.entries {
            ImageId::ensure_next_after(entry.id);

            if entry.kind != "image" {
                // Graceful handling of unknown resource.
                continue;
            }

            let Some(bytes) = self.resource_bytes.get(&entry.id) else {
                continue;
            };

            if let Ok(asset) = decode_image_asset(bytes.clone(), entry.name.clone()) {
                let id = ImageId::from_raw(entry.id);
                assets.register_image_with_id(id, asset);
                image_ids.insert(entry.id, id);
            }
        }

        let mut layers = self.layers;
        let inner = self
            .scene
            .layers
            .into_iter()
            .filter_map(|entry| {
                LayerId::ensure_next_after(entry.id);
                let payload = layers.remove(&entry.id)?;
                Some(layer::layer_from_wire(entry, payload, |id| {
                    image_ids.get(&id).copied()
                }))
            })
            .collect();

        Scene::from_layers_with_assets(inner, assets)
    }
}

/// Serializes a `Document` into `.hexmap` archive bytes.
pub fn serialize(document: &Document) -> Result<Vec<u8>, LoadError> {
    let mut files = BTreeMap::new();

    files.insert(
        METADATA_FILE.to_string(),
        serde_json::to_vec_pretty(&document.metadata)?,
    );
    files.insert(
        document.metadata.scene_file.clone(),
        serde_json::to_vec_pretty(&document.scene)?,
    );
    files.insert(
        document.metadata.resources_file.clone(),
        serde_json::to_vec_pretty(&document.resources)?,
    );

    for entry in &document.scene.layers {
        let payload = document
            .layers
            .get(&entry.id)
            .ok_or_else(|| LoadError::MissingEntry(entry.file.clone()))?;
        files.insert(entry.file.clone(), payload.to_bytes()?);
    }

    for resource in &document.resources.entries {
        let bytes = document
            .resource_bytes
            .get(&resource.id)
            .ok_or_else(|| LoadError::MissingEntry(resource.file.clone()))?;
        files.insert(resource.file.clone(), bytes.clone());
    }

    Ok(archive::write_archive(&files)?)
}

/// Parses `.hexmap` archive bytes into a [`Document`].
pub fn deserialize(bytes: &[u8]) -> Result<Document, LoadError> {
    let files = archive::read_archive(bytes)?;

    let metadata_bytes = files.get(METADATA_FILE).ok_or(LoadError::NotAProjectFile)?;
    let metadata: MetadataV1 =
        serde_json::from_slice(metadata_bytes).map_err(|_| LoadError::NotAProjectFile)?;

    if metadata.format != FORMAT_ID {
        return Err(LoadError::NotAProjectFile);
    }
    if metadata.version > CURRENT_VERSION {
        return Err(LoadError::UnsupportedVersion(metadata.version));
    }

    let scene_bytes = files
        .get(&metadata.scene_file)
        .ok_or_else(|| LoadError::MissingEntry(metadata.scene_file.clone()))?;
    let scene: SceneManifestV1 = serde_json::from_slice(scene_bytes)?;

    let resources: ResourceManifestV1 = match files.get(&metadata.resources_file) {
        Some(bytes) => serde_json::from_slice(bytes)?,
        None => ResourceManifestV1::default(),
    };

    let mut layers = BTreeMap::new();
    for entry in &scene.layers {
        // A layer manifest entry pointing at a missing file is treated the
        // same as an unrecognised layer kind: skipped, not a hard error.
        if let Some(bytes) = files.get(&entry.file) {
            layers.insert(entry.id, LayerPayload::parse(&entry.kind, bytes));
        }
    }

    let mut resource_bytes = BTreeMap::new();
    for resource in &resources.entries {
        if let Some(bytes) = files.get(&resource.file) {
            resource_bytes.insert(resource.id, bytes.clone());
        }
    }

    Ok(Document {
        metadata,
        scene,
        layers,
        resources,
        resource_bytes,
    })
}
