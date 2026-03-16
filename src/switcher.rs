//! Version switching module
//!
//! Implements tool version switching via atomic symlink updates.
//! Updates `~/.vex/current/<tool>` and executable links in `~/.vex/bin/`.

use crate::config;
use crate::error::{Result, VexError};
use crate::tools::Tool;
use owo_colors::OwoColorize;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use uuid::Uuid;

fn vex_dir() -> Result<PathBuf> {
    config::vex_home().ok_or(VexError::HomeDirectoryNotFound)
}

/// Switch tool to specified version
///
/// Atomically updates `~/.vex/current/<tool>` symlink and executable links in `~/.vex/bin/`.
///
/// # Arguments
/// - `tool` - Tool implementation
/// - `version` - Target version number (must be installed)
///
/// # Errors
/// - `VexError::VersionNotFound` - Version not installed
/// - `VexError::Io` - Symlink operation failed
pub fn switch_version(tool: &dyn Tool, version: &str) -> Result<()> {
    info!("Switching version: {}@{}", tool.name(), version);
    switch_version_in(tool, version, &vex_dir()?)
}

fn switch_version_in(tool: &dyn Tool, version: &str, base_dir: &Path) -> Result<()> {
    debug!("Switch version in base_dir: {}", base_dir.display());
    let toolchain_dir = base_dir.join("toolchains").join(tool.name()).join(version);

    if !toolchain_dir.exists() {
        return Err(VexError::VersionNotFound {
            tool: tool.name().to_string(),
            version: version.to_string(),
            suggestions: String::new(),
        });
    }

    // Save current version for rollback
    let current_dir = base_dir.join("current");
    let current_link = current_dir.join(tool.name());
    let old_version = if current_link.exists() {
        fs::read_link(&current_link)
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
    } else {
        None
    };

    debug!("Current version: {:?}", old_version);

    println!(
        "{} {} to version {}...",
        "Switching".cyan(),
        tool.name().yellow(),
        version.yellow()
    );

    // Attempt version switch with rollback on failure
    match perform_switch(tool, version, base_dir, &toolchain_dir) {
        Ok(_) => {
            println!("{} Switched to {}@{}", "✓".green(), tool.name(), version);
            Ok(())
        }
        Err(e) => {
            warn!("Version switch failed: {}, attempting rollback", e);

            // Attempt rollback to previous version
            if let Some(ref prev_version) = old_version {
                eprintln!(
                    "{} Version switch failed, rolling back to {}...",
                    "⚠".yellow(),
                    prev_version
                );

                let prev_toolchain_dir = base_dir
                    .join("toolchains")
                    .join(tool.name())
                    .join(prev_version);

                if prev_toolchain_dir.exists() {
                    match perform_switch(tool, prev_version, base_dir, &prev_toolchain_dir) {
                        Ok(_) => {
                            eprintln!(
                                "{} Rolled back to {}@{}",
                                "✓".green(),
                                tool.name(),
                                prev_version
                            );
                        }
                        Err(rollback_err) => {
                            warn!("Rollback also failed: {}", rollback_err);
                            eprintln!("{} Rollback failed: {}", "✗".red(), rollback_err);
                        }
                    }
                } else {
                    warn!("Previous version {} no longer exists", prev_version);
                }
            }

            Err(e)
        }
    }
}

/// Perform the actual version switch operation
fn perform_switch(
    tool: &dyn Tool,
    _version: &str,
    base_dir: &Path,
    toolchain_dir: &Path,
) -> Result<()> {
    // 1. Update current/ symlink
    let current_dir = base_dir.join("current");
    fs::create_dir_all(&current_dir)?;

    let current_link = current_dir.join(tool.name());

    // Use UUID v4 for random temporary filename to prevent TOCTOU race conditions
    let temp_link = current_dir.join(format!(".{}.tmp.{}", tool.name(), Uuid::new_v4()));

    // Verify target directory ownership matches current user (security check)
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let current_uid = unsafe { libc::getuid() };
        // Use fstat (file descriptor-based) instead of stat to prevent TOCTOU race
        let file = fs::File::open(toolchain_dir)?;
        let metadata = file.metadata()?;
        if metadata.uid() != current_uid {
            return Err(VexError::Parse(format!(
                "Security: toolchain directory owner mismatch (expected uid {}, got {})",
                current_uid,
                metadata.uid()
            )));
        }
    }

    let _ = fs::remove_file(&temp_link);
    unix_fs::symlink(toolchain_dir, &temp_link)?;
    fs::rename(&temp_link, &current_link)?;

    // 2. Update executable links in bin/
    let bin_dir = base_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;

    // First, collect all binaries that should exist for this tool version
    let mut new_binaries = std::collections::HashSet::new();

    // Get the list of binaries to link
    let bin_paths = tool.bin_paths();

    // For each expected binary, check if it exists before creating symlink
    for (bin_name, subpath) in &bin_paths {
        let bin_link = bin_dir.join(bin_name);
        let target = toolchain_dir.join(subpath).join(bin_name);

        // Only create symlink if the target binary actually exists
        if target.exists() {
            let _ = fs::remove_file(&bin_link);
            unix_fs::symlink(&target, &bin_link)?;
            new_binaries.insert(bin_name.to_string());
        }
    }

    // Additionally, scan the actual bin directory for any binaries not in bin_paths
    // This handles cases like corepack in Node.js 24 which exists but isn't in bin_names()
    for (_bin_name, subpath) in &bin_paths {
        let actual_bin_dir = toolchain_dir.join(subpath);
        if actual_bin_dir.exists() {
            if let Ok(entries) = fs::read_dir(&actual_bin_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let filename = entry.file_name();
                    let filename_str = filename.to_string_lossy();

                    // Skip if already handled by bin_paths
                    if bin_paths.iter().any(|(name, _)| *name == filename_str) {
                        continue;
                    }

                    // Check if it's an executable file or symlink
                    if let Ok(metadata) = entry.metadata() {
                        let is_executable = if metadata.is_symlink() {
                            // For symlinks, check if the target exists
                            entry.path().exists()
                        } else {
                            // For regular files, check execute permission
                            use std::os::unix::fs::PermissionsExt;
                            (metadata.permissions().mode() & 0o111) != 0
                        };

                        if is_executable {
                            let bin_link = bin_dir.join(&filename);
                            let target = entry.path();
                            let _ = fs::remove_file(&bin_link);
                            unix_fs::symlink(&target, &bin_link)?;
                            new_binaries.insert(filename_str.to_string());
                        }
                    }
                }
            }
        }
    }

    // Clean up old symlinks that point to this tool but no longer exist in the new version
    if let Ok(entries) = fs::read_dir(&bin_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(target) = fs::read_link(entry.path()) {
                let target_str = target.to_string_lossy();
                // Check if this symlink points to this tool's toolchain
                if target_str.contains(&format!("/toolchains/{}/", tool.name())) {
                    let filename = entry.file_name().to_string_lossy().to_string();
                    // If this binary is not in the new version, remove it
                    if !new_binaries.contains(&filename) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::go::GoTool;
    use crate::tools::node::NodeTool;
    use crate::tools::rust::RustTool;

    pub(crate) fn make_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vex_switcher_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_switch_version_not_found() {
        let base = make_temp_dir("not_found");
        let result = switch_version_in(&NodeTool, "99.99.99", &base);
        assert!(result.is_err());
        if let Err(VexError::VersionNotFound {
            tool,
            version,
            suggestions: _,
        }) = result
        {
            assert_eq!(tool, "node");
            assert_eq!(version, "99.99.99");
        } else {
            panic!("Expected VersionNotFound error");
        }
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_creates_current_symlink() {
        let base = make_temp_dir("current_link");

        // Create fake toolchain directory
        let tc = base.join("toolchains/node/20.11.0/bin");
        fs::create_dir_all(&tc).unwrap();
        // Create fake binary files
        for name in &["node", "npm", "npx"] {
            fs::write(tc.join(name), "fake").unwrap();
        }

        let result = switch_version_in(&NodeTool, "20.11.0", &base);
        assert!(result.is_ok());

        // Verify current/node symlink exists and points correctly
        let current_link = base.join("current/node");
        assert!(current_link.exists());
        let target = fs::read_link(&current_link).unwrap();
        assert!(target.ends_with("toolchains/node/20.11.0"));

        // Verify links in bin/
        assert!(base.join("bin/node").exists());
        assert!(base.join("bin/npm").exists());
        assert!(base.join("bin/npx").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_go_creates_correct_links() {
        let base = make_temp_dir("go_links");

        let tc = base.join("toolchains/go/1.23.5/bin");
        fs::create_dir_all(&tc).unwrap();
        fs::write(tc.join("go"), "fake").unwrap();
        fs::write(tc.join("gofmt"), "fake").unwrap();

        let result = switch_version_in(&GoTool, "1.23.5", &base);
        assert!(result.is_ok());

        assert!(base.join("bin/go").exists());
        assert!(base.join("bin/gofmt").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_rust_separate_bin_paths() {
        let base = make_temp_dir("rust_paths");

        // Rust components in different subdirectories
        let rustc_dir = base.join("toolchains/rust/1.93.1/rustc/bin");
        let cargo_dir = base.join("toolchains/rust/1.93.1/cargo/bin");
        let rustfmt_dir = base.join("toolchains/rust/1.93.1/rustfmt-preview/bin");
        let clippy_dir = base.join("toolchains/rust/1.93.1/clippy-preview/bin");
        let analyzer_dir = base.join("toolchains/rust/1.93.1/rust-analyzer-preview/bin");
        fs::create_dir_all(&rustc_dir).unwrap();
        fs::create_dir_all(&cargo_dir).unwrap();
        fs::create_dir_all(&rustfmt_dir).unwrap();
        fs::create_dir_all(&clippy_dir).unwrap();
        fs::create_dir_all(&analyzer_dir).unwrap();
        fs::write(rustc_dir.join("rustc"), "fake").unwrap();
        fs::write(rustc_dir.join("rustdoc"), "fake").unwrap();
        fs::write(rustc_dir.join("rust-gdb"), "fake").unwrap();
        fs::write(rustc_dir.join("rust-gdbgui"), "fake").unwrap();
        fs::write(rustc_dir.join("rust-lldb"), "fake").unwrap();
        fs::write(cargo_dir.join("cargo"), "fake").unwrap();
        fs::write(rustfmt_dir.join("rustfmt"), "fake").unwrap();
        fs::write(rustfmt_dir.join("cargo-fmt"), "fake").unwrap();
        fs::write(clippy_dir.join("cargo-clippy"), "fake").unwrap();
        fs::write(clippy_dir.join("clippy-driver"), "fake").unwrap();
        fs::write(analyzer_dir.join("rust-analyzer"), "fake").unwrap();

        let result = switch_version_in(&RustTool, "1.93.1", &base);
        assert!(result.is_ok());

        // Verify bin links point to different subdirectories
        let rustc_target = fs::read_link(base.join("bin/rustc")).unwrap();
        let cargo_target = fs::read_link(base.join("bin/cargo")).unwrap();
        assert!(rustc_target.to_string_lossy().contains("rustc/bin/rustc"));
        assert!(cargo_target.to_string_lossy().contains("cargo/bin/cargo"));

        // Verify clippy, rustfmt, rust-analyzer links
        assert!(base.join("bin/rustfmt").exists());
        assert!(base.join("bin/cargo-fmt").exists());
        assert!(base.join("bin/cargo-clippy").exists());
        assert!(base.join("bin/clippy-driver").exists());
        assert!(base.join("bin/rust-analyzer").exists());
        assert!(base.join("bin/rustdoc").exists());
        assert!(base.join("bin/rust-gdb").exists());
        assert!(base.join("bin/rust-gdbgui").exists());
        assert!(base.join("bin/rust-lldb").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_replaces_existing_links() {
        let base = make_temp_dir("replace_links");

        // First create v1
        let tc_v1 = base.join("toolchains/node/1.0.0/bin");
        fs::create_dir_all(&tc_v1).unwrap();
        for name in &["node", "npm", "npx"] {
            fs::write(tc_v1.join(name), "v1").unwrap();
        }
        switch_version_in(&NodeTool, "1.0.0", &base).unwrap();

        // Then switch to v2
        let tc_v2 = base.join("toolchains/node/2.0.0/bin");
        fs::create_dir_all(&tc_v2).unwrap();
        for name in &["node", "npm", "npx"] {
            fs::write(tc_v2.join(name), "v2").unwrap();
        }
        switch_version_in(&NodeTool, "2.0.0", &base).unwrap();

        // current should point to v2
        let target = fs::read_link(base.join("current/node")).unwrap();
        assert!(target.ends_with("toolchains/node/2.0.0"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_dynamic_binary_detection() {
        use std::os::unix::fs::PermissionsExt;
        let base = make_temp_dir("dynamic_bin");

        // Create Node.js 24 with corepack
        let tc_v24 = base.join("toolchains/node/24.0.0/bin");
        fs::create_dir_all(&tc_v24).unwrap();
        for name in &["node", "npm", "npx", "corepack"] {
            let path = tc_v24.join(name);
            fs::write(&path, "v24").unwrap();
            // Make executable
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
        switch_version_in(&NodeTool, "24.0.0", &base).unwrap();

        // Verify corepack link exists
        assert!(base.join("bin/corepack").exists());

        // Create Node.js 25 without corepack
        let tc_v25 = base.join("toolchains/node/25.0.0/bin");
        fs::create_dir_all(&tc_v25).unwrap();
        for name in &["node", "npm", "npx"] {
            let path = tc_v25.join(name);
            fs::write(&path, "v25").unwrap();
            // Make executable
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
        switch_version_in(&NodeTool, "25.0.0", &base).unwrap();

        // Verify corepack link is removed
        assert!(!base.join("bin/corepack").exists());
        // Verify other links still exist
        assert!(base.join("bin/node").exists());
        assert!(base.join("bin/npm").exists());
        assert!(base.join("bin/npx").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_cleanup_stale_symlinks() {
        use std::os::unix::fs::PermissionsExt;
        let base = make_temp_dir("cleanup_stale");

        // Create v1 with extra binary
        let tc_v1 = base.join("toolchains/node/1.0.0/bin");
        fs::create_dir_all(&tc_v1).unwrap();
        for name in &["node", "npm", "npx", "extra-tool"] {
            let path = tc_v1.join(name);
            fs::write(&path, "v1").unwrap();
            // Make executable
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
        switch_version_in(&NodeTool, "1.0.0", &base).unwrap();

        // Verify extra-tool link exists
        assert!(base.join("bin/extra-tool").exists());

        // Switch to v2 without extra-tool
        let tc_v2 = base.join("toolchains/node/2.0.0/bin");
        fs::create_dir_all(&tc_v2).unwrap();
        for name in &["node", "npm", "npx"] {
            let path = tc_v2.join(name);
            fs::write(&path, "v2").unwrap();
            // Make executable
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
        switch_version_in(&NodeTool, "2.0.0", &base).unwrap();

        // Verify extra-tool link is cleaned up
        assert!(!base.join("bin/extra-tool").exists());
        // Verify standard links still exist
        assert!(base.join("bin/node").exists());
        assert!(base.join("bin/npm").exists());
        assert!(base.join("bin/npx").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_version_empty_toolchain() {
        let base = make_temp_dir("empty_toolchain");

        // Create toolchain directory but no binaries
        let tc = base.join("toolchains/node/20.0.0/bin");
        fs::create_dir_all(&tc).unwrap();

        let result = switch_version_in(&NodeTool, "20.0.0", &base);
        // Should succeed even with no binaries
        assert!(result.is_ok());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_version_creates_bin_directory() {
        let base = make_temp_dir("create_bin_dir");

        let tc = base.join("toolchains/go/1.21.0/bin");
        fs::create_dir_all(&tc).unwrap();
        fs::write(tc.join("go"), "fake").unwrap();

        // Ensure bin directory doesn't exist
        let bin_dir = base.join("bin");
        assert!(!bin_dir.exists());

        let result = switch_version_in(&GoTool, "1.21.0", &base);
        assert!(result.is_ok());

        // Verify bin directory was created
        assert!(bin_dir.exists());
        assert!(bin_dir.is_dir());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_version_symlink_atomicity() {
        use std::os::unix::fs::PermissionsExt;
        let base = make_temp_dir("atomicity");

        // Create two versions
        let tc_v1 = base.join("toolchains/node/1.0.0/bin");
        let tc_v2 = base.join("toolchains/node/2.0.0/bin");
        fs::create_dir_all(&tc_v1).unwrap();
        fs::create_dir_all(&tc_v2).unwrap();

        for name in &["node", "npm"] {
            let path1 = tc_v1.join(name);
            fs::write(&path1, "v1").unwrap();
            let mut perms = fs::metadata(&path1).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path1, perms).unwrap();

            let path2 = tc_v2.join(name);
            fs::write(&path2, "v2").unwrap();
            let mut perms = fs::metadata(&path2).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path2, perms).unwrap();
        }

        // Switch to v1
        switch_version_in(&NodeTool, "1.0.0", &base).unwrap();
        let target1 = fs::read_link(base.join("current/node")).unwrap();

        // Switch to v2
        switch_version_in(&NodeTool, "2.0.0", &base).unwrap();
        let target2 = fs::read_link(base.join("current/node")).unwrap();

        // Verify targets are different
        assert_ne!(target1, target2);
        assert!(target2.ends_with("toolchains/node/2.0.0"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_version_with_special_characters() {
        use std::os::unix::fs::PermissionsExt;
        let base = make_temp_dir("special_chars");

        // Create version with special characters (e.g., beta, rc)
        let tc = base.join("toolchains/node/20.0.0-beta.1/bin");
        fs::create_dir_all(&tc).unwrap();
        for name in &["node", "npm"] {
            let path = tc.join(name);
            fs::write(&path, "beta").unwrap();
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }

        let result = switch_version_in(&NodeTool, "20.0.0-beta.1", &base);
        assert!(result.is_ok());

        let target = fs::read_link(base.join("current/node")).unwrap();
        assert!(target.to_string_lossy().contains("20.0.0-beta.1"));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_multiple_tools_independently() {
        use std::os::unix::fs::PermissionsExt;
        let base = make_temp_dir("multi_tools");

        // Create Node.js toolchain
        let node_tc = base.join("toolchains/node/20.0.0/bin");
        fs::create_dir_all(&node_tc).unwrap();
        let node_path = node_tc.join("node");
        fs::write(&node_path, "node").unwrap();
        let mut perms = fs::metadata(&node_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&node_path, perms).unwrap();

        // Create Go toolchain
        let go_tc = base.join("toolchains/go/1.21.0/bin");
        fs::create_dir_all(&go_tc).unwrap();
        let go_path = go_tc.join("go");
        fs::write(&go_path, "go").unwrap();
        let mut perms = fs::metadata(&go_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&go_path, perms).unwrap();

        // Switch both
        switch_version_in(&NodeTool, "20.0.0", &base).unwrap();
        switch_version_in(&GoTool, "1.21.0", &base).unwrap();

        // Verify both are active
        assert!(base.join("current/node").exists());
        assert!(base.join("current/go").exists());
        assert!(base.join("bin/node").exists());
        assert!(base.join("bin/go").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_version_preserves_other_tools() {
        use std::os::unix::fs::PermissionsExt;
        let base = make_temp_dir("preserve_tools");

        // Setup Node.js
        let node_tc = base.join("toolchains/node/20.0.0/bin");
        fs::create_dir_all(&node_tc).unwrap();
        let node_path = node_tc.join("node");
        fs::write(&node_path, "node").unwrap();
        let mut perms = fs::metadata(&node_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&node_path, perms).unwrap();

        // Setup Go
        let go_tc = base.join("toolchains/go/1.21.0/bin");
        fs::create_dir_all(&go_tc).unwrap();
        let go_path = go_tc.join("go");
        fs::write(&go_path, "go").unwrap();
        let mut perms = fs::metadata(&go_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&go_path, perms).unwrap();

        // Activate both
        switch_version_in(&NodeTool, "20.0.0", &base).unwrap();
        switch_version_in(&GoTool, "1.21.0", &base).unwrap();

        // Switch Node.js to different version
        let node_tc2 = base.join("toolchains/node/21.0.0/bin");
        fs::create_dir_all(&node_tc2).unwrap();
        let node_path2 = node_tc2.join("node");
        fs::write(&node_path2, "node21").unwrap();
        let mut perms = fs::metadata(&node_path2).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&node_path2, perms).unwrap();

        switch_version_in(&NodeTool, "21.0.0", &base).unwrap();

        // Verify Go is still active
        assert!(base.join("current/go").exists());
        assert!(base.join("bin/go").exists());

        // Verify Node.js was updated
        let node_target = fs::read_link(base.join("current/node")).unwrap();
        assert!(node_target.to_string_lossy().contains("21.0.0"));

        let _ = fs::remove_dir_all(&base);
    }
}
