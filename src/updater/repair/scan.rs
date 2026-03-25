use crate::error::Result;
use std::fs;
use std::path::Path;

pub(super) type BrokenVersion = (String, String);

pub(super) fn scan_broken_versions(toolchains_dir: &Path) -> Result<Vec<BrokenVersion>> {
    let mut broken_versions = Vec::new();

    for tool_entry in fs::read_dir(toolchains_dir)? {
        let tool_entry = tool_entry?;
        let tool_path = tool_entry.path();
        if !tool_path.is_dir() {
            continue;
        }

        let tool_name = tool_entry.file_name().to_string_lossy().to_string();
        for version_entry in fs::read_dir(&tool_path)? {
            let version_entry = version_entry?;
            let version_path = version_entry.path();
            if !version_path.is_dir() || !has_broken_bin_entries(&version_path)? {
                continue;
            }

            broken_versions.push((
                tool_name.clone(),
                version_entry.file_name().to_string_lossy().to_string(),
            ));
        }
    }

    Ok(broken_versions)
}

fn has_broken_bin_entries(version_path: &Path) -> Result<bool> {
    let bin_dir = version_path.join("bin");
    if !bin_dir.exists() {
        return Ok(false);
    }

    for bin_entry in fs::read_dir(&bin_dir)? {
        let bin_entry = bin_entry?;
        let bin_path = bin_entry.path();
        if !bin_path.is_file() {
            continue;
        }

        if fs::metadata(&bin_path).map(|metadata| metadata.len() == 0)? {
            return Ok(true);
        }
    }

    Ok(false)
}
