//! In memory zip read/write.
//!
//! This module knows nothing about `.hexmap`'s file layout,
//! See `infrastructure::schema`.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

#[derive(Debug)]
pub enum ArchiveError {
    Zip(zip::result::ZipError),
    Io(std::io::Error),
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveError::Zip(err) => write!(f, "archive is corrupted: {err}"),
            ArchiveError::Io(err) => write!(f, "archive I/O error: {err}"),
        }
    }
}

impl std::error::Error for ArchiveError {}

impl From<zip::result::ZipError> for ArchiveError {
    fn from(err: zip::result::ZipError) -> Self {
        ArchiveError::Zip(err)
    }
}

impl From<std::io::Error> for ArchiveError {
    fn from(err: std::io::Error) -> Self {
        ArchiveError::Io(err)
    }
}

/// Reads every file entry of a zip archive into memory,
/// keyed by its path within the archive (e.g. `"layers/1.json"`).
pub fn read_archive(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, ArchiveError> {
    let mut zip = ZipArchive::new(Cursor::new(bytes))?;
    let mut files = BTreeMap::new();

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();
        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf)?;
        files.insert(name, buf);
    }

    Ok(files)
}

/// Writes a set of in-memory files out as zip bytes.
///
/// Save order is deterministic, based on path order and timestamps.
pub fn write_archive(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, ArchiveError> {
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());

    for (name, data) in files {
        zip.start_file(name, options)?;
        zip.write_all(data)?;
    }

    zip.finish()?;
    Ok(buffer.into_inner())
}
