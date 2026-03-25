use super::state::current_version_for_tool;
use crate::commands::versions::{InstalledVersionEntry, InstalledVersionsReport};
use crate::config;
use crate::error::{Result, VexError};
use std::fs;

pub(super) fn collect_installed_versions(tool_name: &str) -> Result<InstalledVersionsReport> {
    let toolchains_dir = config::toolchains_dir()
        .ok_or(VexError::HomeDirectoryNotFound)?
        .join(tool_name);
    let current_version = current_version_for_tool(tool_name);

    if !toolchains_dir.exists() {
        return Ok(InstalledVersionsReport {
            tool: tool_name.to_string(),
            current_version,
            versions: Vec::new(),
        });
    }

    let mut versions: Vec<String> = fs::read_dir(&toolchains_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_type()
                .ok()
                .map(|ft| ft.is_dir())
                .unwrap_or(false)
        })
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    versions.sort();

    Ok(InstalledVersionsReport {
        tool: tool_name.to_string(),
        current_version: current_version.clone(),
        versions: versions
            .into_iter()
            .map(|version| InstalledVersionEntry {
                is_current: current_version.as_ref() == Some(&version),
                version,
            })
            .collect(),
    })
}
