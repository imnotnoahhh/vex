use super::{write::create_symlink, EntryData};
use crate::error::{Result, VexError};
use flate2::read::GzDecoder;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tar::Archive;

pub(super) fn collect_entries(
    archive: &mut Archive<GzDecoder<fs::File>>,
    extract_dir: &Path,
) -> Result<Vec<EntryData>> {
    let mut entries = Vec::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        validate_archive_path(&path)?;

        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() {
            let link_name = entry
                .link_name()?
                .ok_or_else(|| VexError::Parse("Symlink without target".to_string()))?;
            validate_symlink_target(extract_dir, &path, &link_name)?;
            create_symlink(extract_dir, &path, &link_name)?;
            continue;
        }

        let is_dir = entry_type.is_dir();
        let mode = entry.header().mode()?;
        let mut data = Vec::new();
        if !is_dir {
            std::io::Read::read_to_end(&mut entry, &mut data)?;
        }

        entries.push(EntryData {
            path,
            is_dir,
            data,
            mode,
        });
    }

    Ok(entries)
}

fn validate_archive_path(path: &Path) -> Result<()> {
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(VexError::Parse(format!(
            "Archive contains unsafe path: {}. Path traversal detected.",
            path.display()
        )));
    }

    if path.is_absolute() {
        return Err(VexError::Parse(format!(
            "Archive contains absolute path: {}. Only relative paths are allowed.",
            path.display()
        )));
    }

    Ok(())
}

fn validate_symlink_target(extract_dir: &Path, path: &Path, link_name: &Path) -> Result<()> {
    if link_name.is_absolute() {
        return Err(VexError::Parse(format!(
            "Archive contains absolute symlink target: {}",
            link_name.display()
        )));
    }

    let symlink_location = extract_dir.join(path);
    let symlink_parent = symlink_location
        .parent()
        .ok_or_else(|| VexError::Parse("Symlink has no parent directory".to_string()))?;
    let resolved_target = symlink_parent.join(link_name);
    let canonical_target = match resolved_target.canonicalize() {
        Ok(path) => path,
        Err(_) => normalize_path(&resolved_target),
    };

    if !canonical_target.starts_with(extract_dir) {
        return Err(VexError::Parse(format!(
            "Archive contains symlink escaping extraction directory: {} -> {}",
            path.display(),
            link_name.display()
        )));
    }

    Ok(())
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            Component::ParentDir => {
                components.pop();
            }
            Component::CurDir => {}
            other => components.push(other),
        }
    }

    components.iter().collect()
}
