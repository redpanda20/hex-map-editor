use std::collections::HashMap;

use iced::advanced::image::Handle;

use crate::domain::id::ImageId;

#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub name: String,
}

#[derive(Debug, Default, Clone)]
pub struct AssetStore {
    images: HashMap<ImageId, (String, Handle)>,
}

impl AssetStore {
    pub fn register_image(&mut self, asset: ImageAsset) -> ImageId {
        let id = ImageId::next();
        let handle = Handle::from_rgba(asset.width, asset.height, asset.data);
        self.images.insert(id, (asset.name, handle));
        id
    }

    pub fn remove_image(&mut self, id: ImageId) -> Option<Handle> {
        self.images.remove(&id).map(|(_, handle)| handle)
    }

    pub fn image_data(&self, id: ImageId) -> Option<&Handle> {
        self.images.get(&id).map(|(_, handle)| handle)
    }

    pub fn image_name(&self, id: ImageId) -> Option<&str> {
        self.images.get(&id).map(|(str, _)| str.as_str())
    }
}
