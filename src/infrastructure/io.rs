use iced::Task;
use iced::futures::FutureExt;
use rfd::AsyncFileDialog;

use crate::domain::Scene;
use crate::domain::assets::{FileAsset, FileKind};
use crate::domain::id::LayerId;
use crate::infrastructure::image_codec::decode_image_asset;
use crate::{app::Message, domain::assets::ImageAsset};

use super::export::ExportFormat;
use super::schema::{self, Document, LoadError};

const DEFAULT_FILE_NAME: &str = "map.hexmap";
const FILE_EXTENSIONS: &[&str] = &["hexmap"];

#[derive(Debug, Clone, Hash)]
pub enum IoProcess<T> {
    Start,
    Cancelled,
    Finished(Result<T, String>),
}

/// Opens a save dialog and writes the current layers to the chosen file.
pub fn save_project_async(layers: &Scene) -> Task<Message> {
    // `name` isn't tracked anywhere yet - a natural hook for a future
    // "project name" field in the UI.
    let document = Document::from_scene(layers, None);

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

pub fn save_bytes_async(bytes: Vec<u8>, default_name: &str, format: ExportFormat) -> Task<Message> {
    use rfd::AsyncFileDialog;

    Task::future(
        AsyncFileDialog::new()
            .add_filter(format.name(), &[format.extension()])
            .set_file_name(default_name)
            .set_title(format!("Export to {}", format.name()))
            .save_file(),
    )
    .then(move |handle| {
        let inner_bytes = bytes.clone();
        match handle {
            Some(file_handle) => Task::perform(write_future(file_handle, inner_bytes), move |content| {
                Message::Export(format, IoProcess::Finished(content))
            }),
            None => Task::done(Message::Export(format, IoProcess::Cancelled)),
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

/// Opens a load dialog and parses the chosen file into an asset.
pub fn load_asset_async(caller: LayerId, kind: FileKind) -> Task<Message> {
    let (filter_name, extensions) = match kind {
        FileKind::Image => ("Image", &["png", "webp"]),
    };
    Task::future(
        AsyncFileDialog::new()
            .add_filter(filter_name, extensions)
            .set_title("Load asset")
            .pick_file()
            .map(move |handle| (caller, handle)),
    )
    .then(move |(caller, handle)| match handle {
        Some(file_handle) => {
            Task::perform(read_image(file_handle), move |content| Message::LoadAsset {
                caller,
                kind,
                process: IoProcess::Finished(content.map(FileAsset::Image)),
            })
        }
        None => Task::done(Message::LoadAsset {
            caller,
            kind,
            process: IoProcess::Cancelled,
        }),
    })
}

async fn read_future(handle: rfd::FileHandle) -> Result<Document, String> {
    let bytes = handle.read().await;
    schema::deserialize(&bytes).map_err(|err: LoadError| err.to_string())
}

async fn write_future(handle: rfd::FileHandle, bytes: Vec<u8>) -> Result<(), String> {
    handle
        .write(bytes.as_slice())
        .await
        .map_err(|err| err.to_string())
}

async fn read_image(handle: rfd::FileHandle) -> Result<ImageAsset, String> {
    let bytes = handle.read().await;
    decode_image_asset(bytes, handle.file_name())
}
