use crate::commands::prune::RemovalCandidate;
use crate::error::Result;
use crate::fs_utils::path_size;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(super) fn unused_toolchain_candidates(
    vex_dir: &Path,
    retained: &HashMap<(String, String), String>,
) -> Result<Vec<RemovalCandidate>> {
    let toolchains_dir = vex_dir.join("toolchains");
    if !toolchains_dir.exists() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for tool_entry in fs::read_dir(&toolchains_dir)?.filter_map(|e| e.ok()) {
        if !tool_entry
            .file_type()
            .ok()
            .map(|ft| ft.is_dir())
            .unwrap_or(false)
        {
            continue;
        }
        let tool = tool_entry.file_name().to_string_lossy().to_string();
        for version_entry in fs::read_dir(tool_entry.path())?.filter_map(|e| e.ok()) {
            if !version_entry
                .file_type()
                .ok()
                .map(|ft| ft.is_dir())
                .unwrap_or(false)
            {
                continue;
            }

            let version = version_entry.file_name().to_string_lossy().to_string();
            if retained.contains_key(&(tool.clone(), version.clone())) {
                continue;
            }

            let path = version_entry.path();
            candidates.push(RemovalCandidate {
                kind: "toolchain".to_string(),
                bytes: path_size(&path),
                path: path.display().to_string(),
            });
        }
    }

    Ok(candidates)
}
