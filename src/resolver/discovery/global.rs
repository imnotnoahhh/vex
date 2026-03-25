use super::files::read_tool_versions_file;
use std::collections::HashMap;

pub(super) fn merge_global_versions(versions: &mut HashMap<String, String>) {
    let Some(global_path) = vex_global_tool_versions() else {
        return;
    };
    if !global_path.is_file() {
        return;
    }

    for (tool, version) in read_tool_versions_file(&global_path) {
        versions.entry(tool).or_insert(version);
    }
}

pub(super) fn vex_global_tool_versions() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|path| path.join(".vex").join("tool-versions"))
}
