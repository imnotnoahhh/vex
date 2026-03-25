use crate::resolver::parse_tool_versions;
use crate::resolver::TOOL_VERSION_FILES;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn read_tool_versions_file(path: &Path) -> HashMap<String, String> {
    let Ok(content) = fs::read_to_string(path) else {
        return HashMap::new();
    };

    parse_tool_versions(&content).into_iter().collect()
}

pub(super) fn find_tool_specific_version_file(dir: &Path, tool_name: &str) -> Option<PathBuf> {
    TOOL_VERSION_FILES
        .iter()
        .filter(|(_, tool)| *tool == tool_name)
        .map(|(file, _)| dir.join(file))
        .find(|path| path.is_file())
}

pub(super) fn read_language_version_file(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let version = content.trim().to_string();
    (!version.is_empty()).then_some(version)
}
