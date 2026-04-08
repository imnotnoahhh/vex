use super::maybe_fail_bin_link;
use crate::error::{Result, VexError};
use crate::tools::Tool;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;
use uuid::Uuid;

pub(super) fn perform_switch(tool: &dyn Tool, base_dir: &Path, toolchain_dir: &Path) -> Result<()> {
    update_current_symlink(tool, base_dir, toolchain_dir)?;
    update_bin_links(tool, base_dir, toolchain_dir)
}

pub(super) fn rebuild_bin_links(
    tool: &dyn Tool,
    base_dir: &Path,
    toolchain_dir: &Path,
) -> Result<()> {
    update_bin_links(tool, base_dir, toolchain_dir)
}

fn update_current_symlink(tool: &dyn Tool, base_dir: &Path, toolchain_dir: &Path) -> Result<()> {
    let current_dir = base_dir.join("current");
    fs::create_dir_all(&current_dir)?;

    let current_link = current_dir.join(tool.name());
    let temp_link = current_dir.join(format!(".{}.tmp.{}", tool.name(), Uuid::new_v4()));

    verify_toolchain_ownership(toolchain_dir)?;

    let _ = fs::remove_file(&temp_link);
    unix_fs::symlink(toolchain_dir, &temp_link)?;
    fs::rename(&temp_link, &current_link)?;
    Ok(())
}

fn verify_toolchain_ownership(toolchain_dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        let current_uid = unsafe { libc::getuid() };
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

    Ok(())
}

fn update_bin_links(tool: &dyn Tool, base_dir: &Path, toolchain_dir: &Path) -> Result<()> {
    let bin_dir = base_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let bin_paths = tool.bin_paths();
    let mut new_binaries = link_declared_binaries(toolchain_dir, &bin_dir, &bin_paths)?;
    link_dynamic_binaries(toolchain_dir, &bin_dir, &bin_paths, &mut new_binaries)?;
    cleanup_stale_bin_links(tool, &bin_dir, &new_binaries);

    Ok(())
}

fn link_declared_binaries(
    toolchain_dir: &Path,
    bin_dir: &Path,
    bin_paths: &[(&str, &str)],
) -> Result<HashSet<String>> {
    let mut new_binaries = HashSet::new();

    for (bin_name, subpath) in bin_paths {
        let bin_link = bin_dir.join(bin_name);
        let target = toolchain_dir.join(subpath).join(bin_name);

        if target.exists() {
            let _ = fs::remove_file(&bin_link);
            maybe_fail_bin_link(bin_name)?;
            unix_fs::symlink(&target, &bin_link)?;
            new_binaries.insert(bin_name.to_string());
        }
    }

    Ok(new_binaries)
}

fn link_dynamic_binaries(
    toolchain_dir: &Path,
    bin_dir: &Path,
    bin_paths: &[(&str, &str)],
    new_binaries: &mut HashSet<String>,
) -> Result<()> {
    for (_bin_name, subpath) in bin_paths {
        let actual_bin_dir = toolchain_dir.join(subpath);
        if !actual_bin_dir.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&actual_bin_dir) {
            for entry in entries.filter_map(|entry| entry.ok()) {
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();

                if bin_paths.iter().any(|(name, _)| *name == filename_str) {
                    continue;
                }

                if let Ok(metadata) = entry.metadata() {
                    let is_executable = if metadata.is_symlink() {
                        entry.path().exists()
                    } else {
                        use std::os::unix::fs::PermissionsExt;
                        (metadata.permissions().mode() & 0o111) != 0
                    };

                    if is_executable {
                        let bin_link = bin_dir.join(&filename);
                        let target = entry.path();
                        let _ = fs::remove_file(&bin_link);
                        maybe_fail_bin_link(&filename_str)?;
                        unix_fs::symlink(&target, &bin_link)?;
                        new_binaries.insert(filename_str.to_string());
                    }
                }
            }
        }
    }

    Ok(())
}

fn cleanup_stale_bin_links(tool: &dyn Tool, bin_dir: &Path, new_binaries: &HashSet<String>) {
    if let Ok(entries) = fs::read_dir(bin_dir) {
        for entry in entries.filter_map(|entry| entry.ok()) {
            if let Ok(target) = fs::read_link(entry.path()) {
                let target_str = target.to_string_lossy();
                if target_str.contains(&format!("/toolchains/{}/", tool.name())) {
                    let filename = entry.file_name().to_string_lossy().to_string();
                    if !new_binaries.contains(&filename) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}
