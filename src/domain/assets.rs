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

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str) -> ImageAsset {
        ImageAsset {
            encoded: vec![9, 9, 9],
            extension: "png".into(),
            data: vec![0; 2 * 2 * 4], // a 2x2 RGBA image
            width: 2,
            height: 2,
            name: name.into(),
        }
    }

    #[test]
    fn register_image_makes_it_retrievable() {
        let mut store = AssetStore::default();
        let id = store.register_image(asset("sprite.png"));

        assert!(store.image_data(id).is_some());
        assert_eq!(store.image_name(id), Some("sprite.png"));
    }

    #[test]
    fn register_image_with_id_uses_the_given_id() {
        let mut store = AssetStore::default();
        let id = ImageId::from_raw(123);

        store.register_image_with_id(id, asset("restored.png"));

        assert_eq!(store.image_name(id), Some("restored.png"));
    }

    #[test]
    fn remove_image_makes_it_unretrievable() {
        let mut store = AssetStore::default();
        let id = store.register_image(asset("sprite.png"));

        assert!(store.remove_image(id).is_some());
        assert!(store.image_data(id).is_none());
        assert_eq!(store.image_name(id), None);
    }

    #[test]
    fn remove_image_of_unknown_id_is_none() {
        let mut store = AssetStore::default();
        assert!(store.remove_image(ImageId::from_raw(u64::MAX)).is_none());
    }

    #[test]
    fn each_registration_gets_a_distinct_id() {
        let mut store = AssetStore::default();
        let a = store.register_image(asset("a.png"));
        let b = store.register_image(asset("b.png"));
        assert_ne!(a, b);
    }

    #[test]
    fn iter_images_yields_every_registered_image_once() {
        let mut store = AssetStore::default();
        let a = store.register_image(asset("a.png"));
        let b = store.register_image(asset("b.png"));

        let mut seen: Vec<ImageId> = store.iter_images().map(|(id, ..)| id).collect();
        seen.sort_by_key(|id| id.raw());
        let mut expected = vec![a, b];
        expected.sort_by_key(|id| id.raw());

        assert_eq!(seen, expected);
    }

    #[test]
    fn iter_images_exposes_name_extension_and_encoded_bytes() {
        let mut store = AssetStore::default();
        let id = store.register_image(asset("sprite.png"));

        let (found_id, name, extension, encoded) = store.iter_images().next().unwrap();
        assert_eq!(found_id, id);
        assert_eq!(name, "sprite.png");
        assert_eq!(extension, "png");
        assert_eq!(encoded, &[9, 9, 9]);
    }
}
