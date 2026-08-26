use std::io::Cursor;

use iced::Task;
use iced::futures::FutureExt;
use image::{EncodableLayout, ImageReader};
use rfd::AsyncFileDialog;

use crate::domain::Scene;
use crate::domain::id::LayerId;
use crate::infrastructure::IoProcess;
use crate::{app::Message, domain::assets::ImageAsset};

use super::schema::{self, LoadError, SceneV1};

const DEFAULT_FILE_NAME: &str = "map.hexmap";
const FILE_EXTENSIONS: &[&str] = &["hexmap"];

/// Opens a save dialog and writes the current layers to the chosen file.
pub fn save_project_async(layers: &Scene) -> Task<Message> {
    let document = SceneV1::from(layers);

    let bytes = match schema::serialize(&document) {
        Ok(bytes) => bytes,
        Err(err) => return Task::done(Message::Save(IoProcess::Finished(Err(err.to_string())))),
    };

    Task::future(
        AsyncFileDialog::new()
            .add_filter("HexMap Project", FILE_EXTENSIONS)
            .set_file_name(DEFAULT_FILE_NAME)
            .set_title("Save Project")
            .save_file(),
    )
    .then(move |handle| {
        let bytes = bytes.clone();
        match handle {
            Some(file_handle) => Task::perform(write_future(file_handle, bytes), |content| {
                Message::Save(IoProcess::Finished(content))
            }),
            None => Task::done(Message::Save(IoProcess::Cancelled)),
        }
    })
}

pub fn save_bytes_async(bytes: Vec<u8>, default_name: &str) -> Task<Message> {
    use rfd::AsyncFileDialog;

    Task::future(
        AsyncFileDialog::new()
            .set_file_name(default_name)
            .set_title("Export to PNG")
            .save_file(),
    )
    .then(move |handle| {
        let inner_bytes = bytes.clone();
        match handle {
            Some(file_handle) => Task::perform(write_future(file_handle, inner_bytes), |content| {
                Message::Export(IoProcess::Finished(content))
            }),
            None => Task::done(Message::Export(IoProcess::Cancelled)),
        }
    })
}

/// Opens a load dialog and parses the chosen file into a save document.
pub fn load_project_async() -> Task<Message> {
    Task::future(
        AsyncFileDialog::new()
            .add_filter("HexMap Project", FILE_EXTENSIONS)
            .set_title("Open Project")
            .pick_file(),
    )
    .then(|handle| match handle {
        Some(file_handle) => Task::perform(read_future(file_handle), |content| {
            Message::Load(IoProcess::Finished(content))
        }),
        None => Task::done(Message::Load(IoProcess::Cancelled)),
    })
}

/// Opens a load dialog and parses the chosen file into an image asset.
pub fn load_image_async(caller: LayerId) -> Task<Message> {
    Task::future(
        AsyncFileDialog::new()
            .add_filter("Image", &["png"])
            .set_title("Load image")
            .pick_file()
            .map(move |handle| (caller, handle)),
    )
    .then(|(caller, handle)| match handle {
        Some(file_handle) => {
            Task::perform(read_image(file_handle), move |content| Message::LoadAsset {
                caller,
                process: IoProcess::Finished(content),
            })
        }
        None => Task::done(Message::LoadAsset {
            caller,
            process: IoProcess::Cancelled,
        }),
    })
}

async fn read_future(handle: rfd::FileHandle) -> Result<SceneV1, String> {
    let bytes = handle.read().await;
    schema::deserialize(&bytes).map_err(|err: LoadError| err.to_string())
}

async fn write_future(handle: rfd::FileHandle, bytes: Vec<u8>) -> Result<(), String> {
    handle
        .write(bytes.as_bytes())
        .await
        .map_err(|err| err.to_string())
}

async fn read_image(handle: rfd::FileHandle) -> Result<ImageAsset, String> {
    let bytes = handle.read().await;
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|err| err.to_string())?
        .decode()
        .map_err(|err| err.to_string())?;

    let width = image.width();
    let height = image.height();

    Ok(ImageAsset {
        data: image.into_bytes(),
        width,
        height,
    })
}
