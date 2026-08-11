use iced::Task;
use image::EncodableLayout;
use rfd::AsyncFileDialog;

use crate::app::Message;
use crate::state::Scene;

use super::schema::{self, LoadError, SceneV1};

const DEFAULT_FILE_NAME: &str = "map.hexmap";
const FILE_EXTENSIONS: &[&str] = &["hexmap"];

/// Opens a save dialog and writes the current layers to the chosen file.
pub fn save_project_async(layers: &Scene) -> Task<Message> {
    let document = SceneV1::from(layers);

    let bytes = match schema::serialize(&document) {
        Ok(bytes) => bytes,
        Err(err) => return Task::done(Message::ProjectSaved(Err(err.to_string()))),
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
            Some(file_handle) => {
                Task::perform(write_future(file_handle, bytes), Message::ProjectSaved)
            }
            None => Task::done(Message::ProjectSaveCancelled),
        }
    })
}

async fn write_future(handle: rfd::FileHandle, bytes: Vec<u8>) -> Result<(), String> {
    handle
        .write(bytes.as_bytes())
        .await
        .map_err(|err| err.to_string())
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
        Some(file_handle) => Task::perform(read_future(file_handle), Message::ProjectLoaded),
        None => Task::done(Message::ProjectLoadCancelled),
    })
}

async fn read_future(handle: rfd::FileHandle) -> Result<SceneV1, String> {
    let bytes = handle.read().await;
    schema::deserialize(&bytes).map_err(|err: LoadError| err.to_string())
}
