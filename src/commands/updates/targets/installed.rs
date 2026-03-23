use crate::config;
use crate::error::{Result, VexError};
use std::fs;

pub(super) fn latest_installed_version(tool_name: &str) -> Result<Option<String>> {
    let tool_dir = config::vex_home()
        .ok_or(VexError::HomeDirectoryNotFound)?
        .join("toolchains")
        .join(tool_name);

    if !tool_dir.exists() {
        return Ok(None);
    }

    let mut versions = fs::read_dir(&tool_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_type()
                .ok()
                .map(|ft| ft.is_dir())
                .unwrap_or(false)
        })
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    versions.sort_by_key(|version| version_sort_key(version));
    Ok(versions.pop())
}

fn version_sort_key(version: &str) -> Vec<u32> {
    version
        .trim_start_matches('v')
        .split('.')
        .filter_map(|segment| segment.parse::<u32>().ok())
        .collect()
}
