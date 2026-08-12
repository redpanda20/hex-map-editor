mod compat;
mod convert;
mod export;
mod io;
pub mod schema;

pub use compat::{Duration, Instant};
pub use export::export_png;
pub use io::{load_project_async, save_bytes_async, save_project_async};
pub use schema::SceneV1;

#[derive(Debug, Clone)]
pub enum IoProcess<T> {
    Start,
    Cancelled,
    Finished(Result<T, String>),
}
