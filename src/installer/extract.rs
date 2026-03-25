mod entries;
mod write;

use crate::error::{Result, VexError};
use flate2::read::GzDecoder;
use std::fs;
use std::path::{Path, PathBuf};
use tar::Archive;

pub(super) struct EntryData {
    path: PathBuf,
    is_dir: bool,
    data: Vec<u8>,
    mode: u32,
}

use entries::collect_entries;
use write::{create_directories, write_files_in_parallel};

pub(super) fn extract_archive(
    archive: &mut Archive<GzDecoder<fs::File>>,
    extract_dir: &Path,
) -> Result<()> {
    let entries = collect_entries(archive, extract_dir)?;
    let (dirs, files): (Vec<_>, Vec<_>) = entries.into_iter().partition(|entry| entry.is_dir);

    create_directories(extract_dir, dirs)?;
    write_files_in_parallel(extract_dir, files)
}

pub(super) fn find_extracted_root(extract_dir: &Path) -> Result<PathBuf> {
    fs::read_dir(extract_dir)?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            entry
                .file_type()
                .ok()
                .map(|kind| kind.is_dir())
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .ok_or_else(|| VexError::Parse("No directory found after extraction".to_string()))
}
