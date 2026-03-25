use crate::error::{Result, VexError};
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::Disks;

/// Check if sufficient disk space is available
///
/// # Arguments
/// - `path` - Path to check disk space for
/// - `required_bytes` - Minimum required bytes
pub(super) fn check_disk_space(path: &Path, required_bytes: u64) -> Result<()> {
    let disks = Disks::new_with_refreshed_list();

    for disk in &disks {
        if path.starts_with(disk.mount_point()) {
            let available = disk.available_space();
            if available < required_bytes {
                return Err(VexError::DiskSpace {
                    need: required_bytes / (1024 * 1024 * 1024),
                    available: available / (1024 * 1024 * 1024),
                });
            }
            return Ok(());
        }
    }

    Ok(())
}

/// Cleanup guard: automatically cleans up temporary files on installation failure (RAII pattern)
///
/// Call `disarm()` after successful installation to disable the guard, otherwise registered temporary paths
/// are automatically deleted on `Drop`.
pub(super) struct CleanupGuard {
    pub(super) paths: Vec<PathBuf>,
    pub(super) disarmed: bool,
}

impl CleanupGuard {
    pub(super) fn new() -> Self {
        Self {
            paths: Vec::new(),
            disarmed: false,
        }
    }

    pub(super) fn add(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    pub(super) fn disarm(&mut self) {
        self.disarmed = true;
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if self.disarmed {
            return;
        }

        for path in &self.paths {
            if path.is_dir() {
                let _ = fs::remove_dir_all(path);
            } else if path.is_file() {
                let _ = fs::remove_file(path);
            }
        }
    }
}
