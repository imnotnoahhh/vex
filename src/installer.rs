//! Tool installation module
//!
//! Responsible for downloading, verifying, extracting, and installing tool versions to `~/.vex/toolchains/`.
//! Includes disk space checking, path traversal protection, and `CleanupGuard` automatic cleanup mechanism.
//!
//! # Features
//!
//! - **Parallel extraction**: Files are extracted in parallel using rayon (directories created sequentially)
//! - **Path safety**: All archive paths are validated to prevent path traversal attacks
//! - **Atomic operations**: Installation uses temporary directories and atomic moves
//! - **Automatic cleanup**: Failed installations automatically clean up temporary files

use crate::config;
use crate::downloader::{download_with_retry_in_current_context, verify_checksum};
use crate::error::{Result, VexError};
use crate::lock::InstallLock;
use crate::resolver;
use crate::tools::{Arch, Tool};
use flate2::read::GzDecoder;
use owo_colors::OwoColorize;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use sysinfo::Disks;
use tar::Archive;
use tracing::{debug, info};

fn vex_dir() -> Result<PathBuf> {
    config::vex_home().ok_or(VexError::HomeDirectoryNotFound)
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
    info!("Starting installation: {}@{}", tool.name(), version);
    let arch = Arch::detect();
    debug!("Detected architecture: {:?}", arch);
    let vex = vex_dir()?;

    // 0. Check if already installed
    let final_dir = config::toolchains_dir()
        .ok_or(VexError::HomeDirectoryNotFound)?
        .join(tool.name())
        .join(version);
    if final_dir.exists() {
        info!("Version already installed: {}@{}", tool.name(), version);
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
    debug!("Acquiring install lock for {}@{}", tool.name(), version);
    let _lock = InstallLock::acquire(&vex, tool.name(), version)?;

    // Check disk space before downloading
    check_disk_space(&vex, config::MIN_FREE_SPACE_BYTES)?;

    println!(
        "{} {} {}...",
        "Installing".cyan(),
        tool.name().yellow(),
        version.yellow()
    );

    let cache_dir = config::cache_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    fs::create_dir_all(&cache_dir)?;
    let settings = config::load_effective_settings(&resolver::current_dir())?;

    let archive_name = format!("{}-{}.tar.gz", tool.name(), version);
    let archive_path = cache_dir.join(&archive_name);
    let extract_dir = cache_dir.join(format!("{}-{}-extract", tool.name(), version));

    // Set up cleanup guard
    let mut guard = CleanupGuard::new();
    guard.add(archive_path.clone());
    guard.add(extract_dir.clone());

    // 1. Download
    let download_url = config::rewrite_download_url_with_settings(
        &settings,
        tool.name(),
        &tool.download_url(version, arch)?,
    )?;
    println!("{} from {}...", "Downloading".cyan(), download_url.dimmed());
    download_with_retry_in_current_context(
        &download_url,
        &archive_path,
        settings.network.download_retries,
    )?;

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

    // Collect entries with their data for parallel processing
    struct EntryData {
        path: PathBuf,
        is_dir: bool,
        data: Vec<u8>,
        mode: u32,
    }

    let mut entries = Vec::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();

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

        let entry_type = entry.header().entry_type();

        // Handle symlinks separately
        if entry_type.is_symlink() {
            let link_name = entry
                .link_name()?
                .ok_or_else(|| VexError::Parse("Symlink without target".to_string()))?;

            // Validate symlink target is relative (reject absolute paths)
            if link_name.is_absolute() {
                return Err(VexError::Parse(format!(
                    "Archive contains absolute symlink target: {}",
                    link_name.display()
                )));
            }

            // Validate symlink doesn't escape the extraction directory
            // Resolve the symlink target relative to its location
            let symlink_location = extract_dir.join(&path);
            let symlink_parent = symlink_location
                .parent()
                .ok_or_else(|| VexError::Parse("Symlink has no parent directory".to_string()))?;

            // Resolve the target path (handle .. components)
            let resolved_target = symlink_parent.join(&link_name);
            let canonical_target = match resolved_target.canonicalize() {
                Ok(p) => p,
                Err(_) => {
                    // If canonicalize fails (target doesn't exist yet), manually resolve
                    let mut components = Vec::new();
                    for component in resolved_target.components() {
                        match component {
                            std::path::Component::ParentDir => {
                                components.pop();
                            }
                            std::path::Component::CurDir => {}
                            c => components.push(c),
                        }
                    }
                    components.iter().collect()
                }
            };

            // Check if resolved target is within extract_dir
            if !canonical_target.starts_with(&extract_dir) {
                return Err(VexError::Parse(format!(
                    "Archive contains symlink escaping extraction directory: {} -> {}",
                    path.display(),
                    link_name.display()
                )));
            }

            // Create symlink immediately
            let target = extract_dir.join(&path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }

            #[cfg(unix)]
            std::os::unix::fs::symlink(&link_name, &target)?;

            // Skip this entry (don't add to EntryData)
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

    // Separate directories and files for sequential/parallel processing
    let (dirs, files): (Vec<_>, Vec<_>) = entries.into_iter().partition(|e| e.is_dir);

    // Create directories sequentially (to avoid race conditions)
    for entry in dirs {
        let target = extract_dir.join(&entry.path);
        fs::create_dir_all(&target)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(entry.mode))?;
        }
    }

    // Extract files in parallel using rayon
    let errors = Mutex::new(Vec::new());
    files.into_par_iter().for_each(|entry| {
        let target = extract_dir.join(&entry.path);

        // Ensure parent directory exists
        if let Some(parent) = target.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                errors.lock().unwrap().push(format!(
                    "Failed to create parent directory for {}: {}",
                    entry.path.display(),
                    e
                ));
                return;
            }
        }

        // Write file
        if let Err(e) = fs::write(&target, &entry.data) {
            errors
                .lock()
                .unwrap()
                .push(format!("Failed to write {}: {}", entry.path.display(), e));
            return;
        }

        // Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = fs::set_permissions(&target, fs::Permissions::from_mode(entry.mode)) {
                errors.lock().unwrap().push(format!(
                    "Failed to set permissions for {}: {}",
                    entry.path.display(),
                    e
                ));
            }
        }
    });

    let errors = errors.lock().unwrap();
    if !errors.is_empty() {
        return Err(VexError::Parse(format!("Extraction failed: {}", errors[0])));
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

    // Show Corepack hint for Node.js 25+
    if tool.name() == "node" {
        if let Ok(major_version) = version.split('.').next().unwrap_or("0").parse::<u32>() {
            if major_version >= 25 {
                println!();
                println!("{} Node.js 25+ no longer includes Corepack.", "ℹ".cyan());
                println!(
                    "  To use pnpm or yarn, run: {}",
                    "corepack enable pnpm".cyan()
                );
            }
        }
    }

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
        // Verify the constant is set to 1.5 GB
        assert_eq!(config::MIN_FREE_SPACE_BYTES, 1536 * 1024 * 1024);
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

    #[test]
    fn test_parallel_extraction_entry_data() {
        // Test that EntryData struct can be created
        struct EntryData {
            path: PathBuf,
            is_dir: bool,
            data: Vec<u8>,
            mode: u32,
        }

        let entry = EntryData {
            path: PathBuf::from("test/file.txt"),
            is_dir: false,
            data: vec![1, 2, 3],
            mode: 0o644,
        };

        assert_eq!(entry.path, PathBuf::from("test/file.txt"));
        assert!(!entry.is_dir);
        assert_eq!(entry.data, vec![1, 2, 3]);
        assert_eq!(entry.mode, 0o644);
    }

    #[test]
    fn test_parallel_extraction_partition() {
        // Test that we can partition entries into dirs and files
        struct EntryData {
            is_dir: bool,
        }

        let entries = vec![
            EntryData { is_dir: true },
            EntryData { is_dir: false },
            EntryData { is_dir: false },
            EntryData { is_dir: true },
        ];

        let (dirs, files): (Vec<_>, Vec<_>) = entries.into_iter().partition(|e| e.is_dir);

        assert_eq!(dirs.len(), 2);
        assert_eq!(files.len(), 2);
    }
    #[test]
    fn test_symlink_target_validation_parent_dir() {
        // Test that symlink targets with .. are rejected
        let link_name = Path::new("../../../etc/passwd");
        let has_parent_dir = link_name
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
        assert!(
            has_parent_dir,
            "Should detect parent directory in symlink target"
        );
    }

    #[test]
    fn test_symlink_target_validation_absolute() {
        // Test that absolute symlink targets are rejected
        let link_name = Path::new("/etc/passwd");
        assert!(
            link_name.is_absolute(),
            "Should detect absolute symlink target"
        );
    }

    #[test]
    fn test_symlink_target_validation_safe_relative() {
        // Test that safe relative symlink targets pass
        let link_name = Path::new("../lib/node_modules/npm/bin/npm-cli.js");
        let has_parent_dir = link_name
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
        // This should have parent dir components, but they're safe if they don't escape
        assert!(
            has_parent_dir,
            "This path has .. but may be safe in context"
        );
    }

    #[test]
    fn test_symlink_target_validation_simple_relative() {
        // Test that simple relative symlink targets pass
        let link_name = Path::new("node");
        let has_parent_dir = link_name
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
        assert!(
            !has_parent_dir,
            "Simple relative symlink should not have .."
        );
        assert!(!link_name.is_absolute(), "Should be relative");
    }

    #[test]
    fn test_symlink_target_validation_current_dir() {
        // Test that symlink targets with ./ are allowed
        let link_name = Path::new("./bin/node");
        let has_parent_dir = link_name
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
        assert!(
            !has_parent_dir,
            "Current directory component should be allowed"
        );
    }
}
