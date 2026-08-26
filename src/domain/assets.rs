use std::collections::HashMap;

use iced::advanced::image::Handle;

use crate::domain::id::ImageId;

#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default, Clone)]
pub struct AssetStore {
    images: HashMap<ImageId, Handle>,
}

impl AssetStore {
    pub fn register_image(&mut self, asset: ImageAsset) -> ImageId {
        let id = ImageId::next();
        let handle = Handle::from_rgba(asset.width, asset.height, asset.data);
        self.images.insert(id, handle);
        id
    }

    pub fn remove_image(&mut self, id: ImageId) -> Option<Handle> {
        self.images.remove(&id)
    }

    pub fn image(&self, id: ImageId) -> Option<&Handle> {
        self.images.get(&id)
    }
}
