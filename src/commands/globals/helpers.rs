use super::{GlobalCliEntry, VersionContext};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::tools::python::PYTHON_BUILD_STANDALONE_INTERNAL_ALIAS;

pub(super) fn push_bin_entries(
    entries: &mut Vec<GlobalCliEntry>,
    tool: &str,
    kind: &str,
    source: &str,
    bin_dir: &Path,
    context: Option<&VersionContext>,
    include_name: impl Fn(&str) -> bool,
) {
    if !bin_dir.exists() {
        return;
    }

    let Ok(read_dir) = fs::read_dir(bin_dir) else {
        return;
    };

    for entry in read_dir.filter_map(|entry| entry.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || !include_name(&name) {
            continue;
        }
        let path = entry.path();
        if is_executable_file(&path) {
            entries.push(entry_from_path(tool, &name, kind, source, &path, context));
        }
    }
}

pub(super) fn entry_from_path(
    tool: &str,
    name: &str,
    kind: &str,
    source: &str,
    path: &Path,
    context: Option<&VersionContext>,
) -> GlobalCliEntry {
    GlobalCliEntry {
        tool: tool.to_string(),
        name: name.to_string(),
        kind: kind.to_string(),
        path: path.display().to_string(),
        source: source.to_string(),
        tool_version: context.map(|context| context.version.clone()),
        version_source: context.map(|context| context.source.clone()),
        version_source_path: context.and_then(|context| context.source_path.clone()),
    }
}

pub(super) fn is_user_python_cli(name: &str) -> bool {
    !(name == "activate"
        || name.starts_with("activate.")
        || name.starts_with("Activate.")
        || name == "python"
        || name.starts_with("python3")
        || name == PYTHON_BUILD_STANDALONE_INTERNAL_ALIAS
        || name == "pip"
        || name.starts_with("pip3"))
}

pub(super) fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if metadata.is_dir() {
        return false;
    }
    metadata.permissions().mode() & 0o111 != 0
}

pub(super) fn find_on_path(name: &str) -> Option<PathBuf> {
    std::env::var("PATH").ok()?.split(':').find_map(|entry| {
        if entry.is_empty() {
            return None;
        }
        let candidate = Path::new(entry).join(name);
        is_executable_file(&candidate).then_some(candidate)
    })
}

pub(super) fn matches_filter(filter: Option<&str>, tool: &str, name: &str) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    let filter = filter.to_ascii_lowercase();
    match filter.as_str() {
        "all" => true,
        "npm" => tool == "node",
        "pip" => tool == "python",
        "cargo" => tool == "rust",
        "maven" | "mvn" => {
            tool == "java" && (name.is_empty() || name.contains("maven") || name == "mvn")
        }
        "gradle" => tool == "java" && (name.is_empty() || name.contains("gradle")),
        _ => filter == tool || (!name.is_empty() && filter == name),
    }
}
