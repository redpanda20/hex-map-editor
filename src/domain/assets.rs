use std::collections::HashMap;

use iced::advanced::image::Handle;

use crate::domain::id::ImageId;

/// A loaded image, ready to be registered into an [`AssetStore`].
///
/// Both the decoded pixels (for rendering) and the original encoded bytes
/// (for lossless persistence - see `infrastructure::schema::resource`) are
/// kept, since re-encoding on every save would be both lossy and wasteful.
#[derive(Debug, Clone)]
pub struct ImageAsset {
    /// Original file bytes, exactly as read from disk/archive.
    pub encoded: Vec<u8>,
    /// Lowercase file extension matching `encoded`'s format (e.g. "png",
    /// "webp"), used to name the resource file on save.
    pub extension: String,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub name: String,
}

#[derive(Debug, Clone)]
struct ImageEntry {
    name: String,
    extension: String,
    encoded: Vec<u8>,
    handle: Handle,
}

#[derive(Debug, Default, Clone)]
pub struct AssetStore {
    images: HashMap<ImageId, ImageEntry>,
}

impl AssetStore {
    /// Registers a new image, assigning it a fresh id.
    pub fn register_image(&mut self, asset: ImageAsset) -> ImageId {
        let id = ImageId::next();
        self.insert_image(id, asset);
        id
    }

    /// Registers an image under a specific id. Used when restoring a saved
    /// project, so that layers referencing the id (persisted as a raw `u64`)
    /// resolve correctly - see `infrastructure::schema::layer::image`.
    pub fn register_image_with_id(&mut self, id: ImageId, asset: ImageAsset) {
        self.insert_image(id, asset);
    }

    fn insert_image(&mut self, id: ImageId, asset: ImageAsset) {
        let handle = Handle::from_rgba(asset.width, asset.height, asset.data);
        self.images.insert(
            id,
            ImageEntry {
                name: asset.name,
                extension: asset.extension,
                encoded: asset.encoded,
                handle,
            },
        );
    }

    pub fn remove_image(&mut self, id: ImageId) -> Option<Handle> {
        self.images.remove(&id).map(|entry| entry.handle)
    }

    pub fn image_data(&self, id: ImageId) -> Option<&Handle> {
        self.images.get(&id).map(|entry| &entry.handle)
    }

    pub fn image_name(&self, id: ImageId) -> Option<&str> {
        self.images.get(&id).map(|entry| entry.name.as_str())
    }

    /// All registered images, keyed by id, for persistence. A given image
    /// may be referenced by more than one layer (a many-to-one relation) -
    /// this yields each registered image exactly once regardless.
    pub fn iter_images(&self) -> impl Iterator<Item = (ImageId, &str, &str, &[u8])> {
        self.images.iter().map(|(id, entry)| {
            (
                *id,
                entry.name.as_str(),
                entry.extension.as_str(),
                entry.encoded.as_slice(),
            )
        })
    }
}
