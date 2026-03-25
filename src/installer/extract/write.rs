use super::EntryData;
use crate::error::{Result, VexError};
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

pub(super) fn create_symlink(extract_dir: &Path, path: &Path, link_name: &Path) -> Result<()> {
    let target = extract_dir.join(path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(link_name, &target)?;

    Ok(())
}

pub(super) fn create_directories(extract_dir: &Path, dirs: Vec<EntryData>) -> Result<()> {
    for entry in dirs {
        let target = extract_dir.join(&entry.path);
        fs::create_dir_all(&target)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(entry.mode))?;
        }
    }

    Ok(())
}

pub(super) fn write_files_in_parallel(extract_dir: &Path, files: Vec<EntryData>) -> Result<()> {
    let errors = Mutex::new(Vec::new());

    files.into_par_iter().for_each(|entry| {
        let target = extract_dir.join(&entry.path);

        if let Some(parent) = target.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                errors.lock().unwrap().push(format!(
                    "Failed to create parent directory for {}: {}",
                    entry.path.display(),
                    error
                ));
                return;
            }
        }

        if let Err(error) = fs::write(&target, &entry.data) {
            errors.lock().unwrap().push(format!(
                "Failed to write {}: {}",
                entry.path.display(),
                error
            ));
            return;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(error) = fs::set_permissions(&target, fs::Permissions::from_mode(entry.mode))
            {
                errors.lock().unwrap().push(format!(
                    "Failed to set permissions for {}: {}",
                    entry.path.display(),
                    error
                ));
            }
        }
    });

    let errors = errors.lock().unwrap();
    if !errors.is_empty() {
        return Err(VexError::Parse(format!("Extraction failed: {}", errors[0])));
    }

    Ok(())
}
