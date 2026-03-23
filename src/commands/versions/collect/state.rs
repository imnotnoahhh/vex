use crate::config;
use crate::versioning::normalize_version;
use std::fs;

pub(super) fn current_version_for_tool(tool_name: &str) -> Option<String> {
    let current_link = config::current_dir()?.join(tool_name);
    fs::read_link(&current_link).ok().and_then(|target| {
        target
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
    })
}

pub(super) fn version_matches_current(current_version: Option<&str>, version: &str) -> bool {
    let normalized = normalize_version(version);
    current_version
        .map(|current| current == normalized || current == version)
        .unwrap_or(false)
}
