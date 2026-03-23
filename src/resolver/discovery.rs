mod cwd;
mod files;
mod global;
mod project;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Traverse upward from start directory to find version mappings for all tools
///
/// `.tool-versions` has higher priority than language-specific files (`.node-version`, etc.).
/// First found version takes precedence (child directory over parent directory).
///
/// # Arguments
/// - `start_dir` - Directory to start searching from
pub fn resolve_versions(start_dir: &Path) -> HashMap<String, String> {
    let mut versions = project::resolve_project_versions(start_dir);
    global::merge_global_versions(&mut versions);
    versions
}

/// Traverse upward from start directory and return only `.tool-versions` entries.
///
/// Child directories take precedence over parents. This intentionally ignores language-specific
/// files and the global `~/.vex/tool-versions` so callers can safely layer local project pins over
/// other inputs such as remote team config.
pub fn resolve_local_tool_versions_only(start_dir: &Path) -> HashMap<String, String> {
    project::collect_tool_versions_from_ancestors(start_dir)
}

/// Traverse upward from start directory and return all project-managed versions.
///
/// This includes `.tool-versions` and language-specific version files, but intentionally excludes
/// the global `~/.vex/tool-versions` so callers can reason about the current project tree only.
pub fn resolve_project_versions(start_dir: &Path) -> HashMap<String, String> {
    project::resolve_project_versions(start_dir)
}

pub fn read_tool_versions_file(path: &Path) -> HashMap<String, String> {
    files::read_tool_versions_file(path)
}

pub fn find_project_source(start_dir: &Path, tool_name: &str) -> Option<PathBuf> {
    project::find_project_source(start_dir, tool_name)
}

#[cfg(test)]
pub fn resolve_version(tool_name: &str, start_dir: &Path) -> Option<String> {
    project::resolve_version(tool_name, start_dir)
}

/// Get current working directory, fallback to "." on failure
pub fn current_dir() -> PathBuf {
    cwd::current_dir()
}
