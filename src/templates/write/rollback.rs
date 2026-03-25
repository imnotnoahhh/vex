use crate::error::{Result, VexError};
use crate::templates::PlannedWrite;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir_in;

#[derive(Debug, Clone)]
pub(in crate::templates) struct AppliedWrite {
    pub(in crate::templates) path: PathBuf,
    pub(in crate::templates) original_contents: Option<Vec<u8>>,
}

pub(in crate::templates) fn apply_write_plan(cwd: &Path, preview: &[PlannedWrite]) -> Result<()> {
    let staging_dir = tempdir_in(cwd).map_err(|e| {
        VexError::Config(format!(
            "Cannot write template files to '{}': {}. Make sure the directory exists and is writable.",
            cwd.display(),
            e
        ))
    })?;
    let staged_paths: Vec<PathBuf> = preview
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let staged_path = staging_dir.path().join(format!("write-{}.tmp", index));
            fs::write(&staged_path, &item.contents)?;
            Ok(staged_path)
        })
        .collect::<Result<_>>()?;

    let mut applied = Vec::new();
    let mut created_dirs = Vec::new();
    for (item, staged_path) in preview.iter().zip(staged_paths.iter()) {
        let original_contents = if item.path.exists() {
            Some(fs::read(&item.path)?)
        } else {
            None
        };

        let write_result = (|| -> Result<()> {
            create_missing_parent_dirs(&item.path, &mut created_dirs)?;
            fs::rename(staged_path, &item.path)?;
            Ok(())
        })();

        if let Err(err) = write_result {
            if let Err(rollback_err) =
                rollback_applied_writes(staging_dir.path(), &applied, &created_dirs)
            {
                return Err(VexError::Config(format!(
                    "Template write failed: {}. Rollback also failed: {}",
                    err, rollback_err
                )));
            }
            return Err(err);
        }

        applied.push(AppliedWrite {
            path: item.path.clone(),
            original_contents,
        });
    }

    Ok(())
}

fn create_missing_parent_dirs(path: &Path, created_dirs: &mut Vec<PathBuf>) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.exists() {
        return Ok(());
    }

    let mut missing = Vec::new();
    let mut current = Some(parent);
    while let Some(dir) = current {
        if dir.exists() {
            break;
        }
        missing.push(dir.to_path_buf());
        current = dir.parent();
    }

    fs::create_dir_all(parent)?;
    for dir in missing {
        if !created_dirs.contains(&dir) {
            created_dirs.push(dir);
        }
    }

    Ok(())
}

pub(in crate::templates) fn rollback_applied_writes(
    staging_dir: &Path,
    applied: &[AppliedWrite],
    created_dirs: &[PathBuf],
) -> Result<()> {
    let mut counter = 0usize;
    let mut errors = Vec::new();

    for item in applied.iter().rev() {
        let result = match &item.original_contents {
            Some(contents) => {
                let staged_path = staging_dir.join(format!("rollback-{}.tmp", counter));
                counter += 1;
                fs::write(&staged_path, contents)?;
                fs::rename(&staged_path, &item.path)
            }
            None => {
                if item.path.exists() {
                    fs::remove_file(&item.path)
                } else {
                    Ok(())
                }
            }
        };

        if let Err(err) = result {
            errors.push(format!("{}: {}", item.path.display(), err));
        }
    }

    for dir in created_dirs {
        if !dir.exists() {
            continue;
        }
        match fs::remove_dir(dir) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) if err.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
            Err(err) => errors.push(format!("{}: {}", dir.display(), err)),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(VexError::Config(format!(
            "Rollback incomplete:\n  - {}",
            errors.join("\n  - ")
        )))
    }
}
