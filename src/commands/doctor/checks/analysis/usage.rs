use crate::commands::doctor::types::{ToolDiskUsage, UnusedVersion};
use crate::error::Result;
use crate::fs_utils::path_size;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(super) fn collect_disk_usage(vex_dir: &Path) -> Result<Vec<ToolDiskUsage>> {
    let toolchains_dir = vex_dir.join("toolchains");
    if !toolchains_dir.exists() {
        return Ok(Vec::new());
    }

    let mut usage = Vec::new();
    for tool_entry in fs::read_dir(&toolchains_dir)?.filter_map(|entry| entry.ok()) {
        if !tool_entry
            .file_type()
            .ok()
            .map(|file_type| file_type.is_dir())
            .unwrap_or(false)
        {
            continue;
        }

        let tool = tool_entry.file_name().to_string_lossy().to_string();
        let mut version_count = 0;
        let mut total_bytes = 0;

        for version_entry in fs::read_dir(tool_entry.path())?.filter_map(|entry| entry.ok()) {
            if version_entry
                .file_type()
                .ok()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
            {
                version_count += 1;
                total_bytes += path_size(&version_entry.path());
            }
        }

        if version_count > 0 {
            usage.push(ToolDiskUsage {
                tool,
                version_count,
                total_bytes,
            });
        }
    }

    usage.sort_by_key(|entry| Reverse(entry.total_bytes));
    Ok(usage)
}

pub(super) fn collect_unused_versions(
    vex_dir: &Path,
    retained: &HashMap<(String, String), String>,
) -> Result<Vec<UnusedVersion>> {
    let toolchains_dir = vex_dir.join("toolchains");
    if !toolchains_dir.exists() {
        return Ok(Vec::new());
    }

    let mut unused = Vec::new();
    for tool_entry in fs::read_dir(&toolchains_dir)?.filter_map(|entry| entry.ok()) {
        if !tool_entry
            .file_type()
            .ok()
            .map(|file_type| file_type.is_dir())
            .unwrap_or(false)
        {
            continue;
        }

        let tool = tool_entry.file_name().to_string_lossy().to_string();
        for version_entry in fs::read_dir(tool_entry.path())?.filter_map(|entry| entry.ok()) {
            if !version_entry
                .file_type()
                .ok()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
            {
                continue;
            }

            let version = version_entry.file_name().to_string_lossy().to_string();
            if !retained.contains_key(&(tool.clone(), version.clone())) {
                let bytes = path_size(&version_entry.path());
                unused.push(UnusedVersion {
                    tool: tool.clone(),
                    version,
                    bytes,
                });
            }
        }
    }

    unused.sort_by_key(|entry| Reverse(entry.bytes));
    Ok(unused)
}
