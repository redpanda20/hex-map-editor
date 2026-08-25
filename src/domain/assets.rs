use std::collections::HashMap;

use crate::domain::id::ImageId;

#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default, Clone)]
pub struct AssetStore {
    images: HashMap<ImageId, ImageAsset>,
}

impl AssetStore {
    pub fn register_image(&mut self, asset: ImageAsset) -> ImageId {
        let id = ImageId::next();
        self.images.insert(id, asset);
        id
    }

    pub fn remove_image(&mut self, id: ImageId) -> Option<ImageAsset> {
        self.images.remove(&id)
    }

    pub fn image(&self, id: ImageId) -> Option<&ImageAsset> {
        self.images.get(&id)
    }
}
