//! Tool installation module
//!
//! Responsible for downloading, verifying, extracting, and installing tool versions to `~/.vex/toolchains/`.
//! Includes disk space checking, path traversal protection, and `CleanupGuard` automatic cleanup mechanism.

use crate::downloader::{download_with_retry, verify_checksum};
use crate::error::{Result, VexError};
use crate::lock::InstallLock;
use crate::tools::{Arch, Tool};
use flate2::read::GzDecoder;
use owo_colors::OwoColorize;
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::Disks;
use tar::Archive;

/// Minimum free disk space before installation (500 MB)
const MIN_FREE_SPACE_BYTES: u64 = 500 * 1024 * 1024;

fn vex_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|p| p.join(".vex"))
        .ok_or(VexError::HomeDirectoryNotFound)
}

/// Check if sufficient disk space is available
///
/// # Arguments
/// - `path` - Path to check disk space for
/// - `required_bytes` - Minimum required bytes
fn check_disk_space(path: &Path, required_bytes: u64) -> Result<()> {
    let disks = Disks::new_with_refreshed_list();

    // Find the disk that contains the path
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

    // If we can't find the disk, proceed anyway (better than failing)
    Ok(())
}

/// Cleanup guard: automatically cleans up temporary files on installation failure (RAII pattern)
///
/// Call `disarm()` after successful installation to disable the guard, otherwise registered temporary paths
/// are automatically deleted on `Drop`.
struct CleanupGuard {
    paths: Vec<PathBuf>,
    disarmed: bool,
}

impl CleanupGuard {
    fn new() -> Self {
        Self {
            paths: Vec::new(),
            disarmed: false,
        }
    }

    fn add(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    fn disarm(&mut self) {
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

/// Install specified tool version
///
/// Complete installation flow: check duplicates → acquire lock → check disk → download → verify → extract → move → post_install.
///
/// # Arguments
/// - `tool` - Tool implementation
/// - `version` - Version number
///
/// # Errors
/// - `VexError::LockConflict` - Another process is installing
/// - `VexError::DiskSpace` - Insufficient disk space
/// - `VexError::Network` - Download failed
/// - `VexError::ChecksumMismatch` - Checksum mismatch
pub fn install(tool: &dyn Tool, version: &str) -> Result<()> {
    let arch = Arch::detect();
    let vex = vex_dir()?;

    // 0. Check if already installed
    let final_dir = vex.join("toolchains").join(tool.name()).join(version);
    if final_dir.exists() {
        println!(
            "{} {} is already installed.",
            format!("{}@{}", tool.name(), version).yellow(),
            "✓".green()
        );
        println!(
            "Use {} to switch to it.",
            format!("'vex use {}@{}'", tool.name(), version).cyan()
        );
        return Ok(());
    }

    // Acquire install lock (fail fast if another process is installing the same version)
    let _lock = InstallLock::acquire(&vex, tool.name(), version)?;

    // Check disk space before downloading
    check_disk_space(&vex, MIN_FREE_SPACE_BYTES)?;

    println!(
        "{} {} {}...",
        "Installing".cyan(),
        tool.name().yellow(),
        version.yellow()
    );

    let cache_dir = vex.join("cache");
    fs::create_dir_all(&cache_dir)?;

    let archive_name = format!("{}-{}.tar.gz", tool.name(), version);
    let archive_path = cache_dir.join(&archive_name);
    let extract_dir = cache_dir.join(format!("{}-{}-extract", tool.name(), version));

    // Set up cleanup guard
    let mut guard = CleanupGuard::new();
    guard.add(archive_path.clone());
    guard.add(extract_dir.clone());

    // 1. Download
    let download_url = tool.download_url(version, arch)?;
    println!("{} from {}...", "Downloading".cyan(), download_url.dimmed());
    download_with_retry(&download_url, &archive_path, 3)?;

    // 2. Verify checksum
    if let Ok(Some(expected)) = tool.get_checksum(version, arch) {
        println!("{}...", "Verifying checksum".cyan());
        verify_checksum(&archive_path, &expected)?;
        println!("{} Checksum verified", "✓".green());
    }

    // 3. Extract
    println!("{}...", "Extracting".cyan());
    fs::create_dir_all(&extract_dir)?;

    let tar_gz = fs::File::open(&archive_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    // Validate and extract entries to prevent path traversal attacks
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        // Check for path traversal attempts (e.g., "../../../etc/passwd")
        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(VexError::Parse(format!(
                "Archive contains unsafe path: {}. Path traversal detected.",
                path.display()
            )));
        }

        // Check for absolute paths
        if path.is_absolute() {
            return Err(VexError::Parse(format!(
                "Archive contains absolute path: {}. Only relative paths are allowed.",
                path.display()
            )));
        }

        // Extract to the designated directory
        entry.unpack_in(&extract_dir)?;
    }

    // 4. Find extracted directory
    let entries = fs::read_dir(&extract_dir)?;
    let extracted_dir = entries
        .filter_map(|e| e.ok())
        .find(|e| e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false))
        .ok_or_else(|| VexError::Parse("No directory found after extraction".to_string()))?
        .path();

    // 5. Move to final location
    let toolchains_dir = vex.join("toolchains").join(tool.name());
    fs::create_dir_all(&toolchains_dir)?;
    fs::rename(&extracted_dir, &final_dir)?;

    // 5.5. Run post-install hook
    tool.post_install(&final_dir, arch)?;

    // 6. Installation successful, disarm cleanup guard and manually clean up temporary files
    guard.disarm();
    let _ = fs::remove_file(&archive_path);
    let _ = fs::remove_dir_all(&extract_dir);

    println!(
        "{} Installed {} {} to {}",
        "✓".green(),
        tool.name().yellow(),
        version.yellow(),
        final_dir.display().to_string().dimmed()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_path_component_validation_parent_dir() {
        // Test that we can detect parent directory components
        let path = Path::new("../../../etc/passwd");
        let has_parent_dir = path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
        assert!(has_parent_dir, "Should detect parent directory component");
    }

    #[test]
    fn test_path_component_validation_safe_relative() {
        // Test that safe relative paths pass
        let path = Path::new("node-v20.11.0/bin/node");
        let has_parent_dir = path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
        assert!(!has_parent_dir, "Safe relative path should not have ..");
        assert!(!path.is_absolute(), "Should be relative");
    }

    #[test]
    fn test_path_component_validation_absolute() {
        // Test that absolute paths are detected
        let path = Path::new("/etc/passwd");
        assert!(path.is_absolute(), "Should detect absolute path");
    }

    #[test]
    fn test_path_component_validation_mixed() {
        // Test path with .. in the middle
        let path = Path::new("foo/../bar/baz");
        let has_parent_dir = path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
        assert!(has_parent_dir, "Should detect .. in middle of path");
    }

    #[test]
    fn test_path_component_validation_current_dir_ok() {
        // Test that current directory (.) is allowed
        let path = Path::new("./foo/bar");
        let has_parent_dir = path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
        assert!(
            !has_parent_dir,
            "Current directory component should be allowed"
        );
    }

    #[test]
    fn test_check_disk_space_sufficient() {
        let temp_dir = TempDir::new().unwrap();
        // Request 1 byte - should always succeed
        let result = check_disk_space(temp_dir.path(), 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_disk_space_insufficient() {
        let temp_dir = TempDir::new().unwrap();
        // Request an impossibly large amount (1 PB)
        let result = check_disk_space(temp_dir.path(), 1024 * 1024 * 1024 * 1024 * 1024);
        assert!(result.is_err());
        if let Err(VexError::DiskSpace { need, available }) = result {
            assert!(need > available);
        } else {
            panic!("Expected DiskSpace error");
        }
    }

    #[test]
    fn test_min_free_space_constant() {
        // Verify the constant is set to 500 MB
        assert_eq!(MIN_FREE_SPACE_BYTES, 500 * 1024 * 1024);
    }

    #[test]
    fn test_cleanup_guard_new() {
        let guard = CleanupGuard::new();
        assert_eq!(guard.paths.len(), 0);
        assert!(!guard.disarmed);
    }

    #[test]
    fn test_cleanup_guard_add() {
        let mut guard = CleanupGuard::new();
        guard.add(PathBuf::from("/tmp/test"));
        assert_eq!(guard.paths.len(), 1);
    }

    #[test]
    fn test_cleanup_guard_disarm() {
        let mut guard = CleanupGuard::new();
        guard.add(PathBuf::from("/tmp/test"));
        guard.disarm();
        assert!(guard.disarmed);
    }

    #[test]
    fn test_cleanup_guard_drop_disarmed() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test").unwrap();

        {
            let mut guard = CleanupGuard::new();
            guard.add(test_file.clone());
            guard.disarm();
        }

        // File should still exist because guard was disarmed
        assert!(test_file.exists());
    }

    #[test]
    fn test_cleanup_guard_drop_armed_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test").unwrap();

        {
            let mut guard = CleanupGuard::new();
            guard.add(test_file.clone());
        }

        // File should be deleted
        assert!(!test_file.exists());
    }

    #[test]
    fn test_cleanup_guard_drop_armed_dir() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("testdir");
        std::fs::create_dir(&test_dir).unwrap();
        std::fs::write(test_dir.join("file.txt"), "test").unwrap();

        {
            let mut guard = CleanupGuard::new();
            guard.add(test_dir.clone());
        }

        // Directory should be deleted
        assert!(!test_dir.exists());
    }

    #[test]
    fn test_vex_dir_success() {
        // This should succeed on normal systems
        let result = vex_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with(".vex"));
    }
}
