use super::files::{
    find_tool_specific_version_file, read_language_version_file, read_tool_versions_file,
};
#[cfg(test)]
use super::global::vex_global_tool_versions;
use crate::resolver::parse_tool_versions;
use crate::resolver::TOOL_VERSION_FILES;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn resolve_project_versions(start_dir: &Path) -> HashMap<String, String> {
    let mut versions = collect_tool_versions_from_ancestors(start_dir);
    let mut dir = start_dir.to_path_buf();

    loop {
        for (file, tool) in TOOL_VERSION_FILES {
            let path = dir.join(file);
            if let Some(version) = read_language_version_file(&path) {
                versions.entry(tool.to_string()).or_insert(version);
            }
        }

        if !dir.pop() {
            break;
        }
    }

    versions
}

pub(super) fn collect_tool_versions_from_ancestors(start_dir: &Path) -> HashMap<String, String> {
    let mut versions = HashMap::new();
    let mut dir = start_dir.to_path_buf();

    loop {
        let tool_versions = dir.join(".tool-versions");
        if tool_versions.is_file() {
            if let Ok(content) = fs::read_to_string(&tool_versions) {
                for (tool, version) in parse_tool_versions(&content) {
                    versions.entry(tool).or_insert(version);
                }
            }
        }

        if !dir.pop() {
            break;
        }
    }

    versions
}

pub(super) fn find_project_source(start_dir: &Path, tool_name: &str) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();

    loop {
        let tool_versions = dir.join(".tool-versions");
        if tool_versions.is_file()
            && read_tool_versions_file(&tool_versions).contains_key(tool_name)
        {
            return Some(tool_versions);
        }

        if let Some(path) = find_tool_specific_version_file(&dir, tool_name) {
            return Some(path);
        }

        if !dir.pop() {
            break;
        }
    }

    None
}

#[cfg(test)]
pub(super) fn resolve_version(tool_name: &str, start_dir: &Path) -> Option<String> {
    let mut dir = start_dir.to_path_buf();

    loop {
        let tool_versions = dir.join(".tool-versions");
        if tool_versions.is_file() {
            if let Some(version) = read_tool_versions_file(&tool_versions)
                .into_iter()
                .find_map(|(tool, version)| (tool == tool_name).then_some(version))
            {
                return Some(version);
            }
        }

        if let Some(path) = find_tool_specific_version_file(&dir, tool_name) {
            if let Some(version) = read_language_version_file(&path) {
                return Some(version);
            }
        }

        if !dir.pop() {
            break;
        }
    }

    if let Some(global_path) = vex_global_tool_versions() {
        return read_tool_versions_file(&global_path)
            .into_iter()
            .find_map(|(tool, version)| (tool == tool_name).then_some(version));
    }

    None
}
