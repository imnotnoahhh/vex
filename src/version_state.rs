use crate::error::Result;
use crate::resolver;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn read_current_versions(vex_dir: &Path) -> Result<HashMap<String, String>> {
    let current_dir = vex_dir.join("current");
    let mut versions = HashMap::new();
    if !current_dir.exists() {
        return Ok(versions);
    }

    for entry in fs::read_dir(&current_dir)?.filter_map(|e| e.ok()) {
        let tool = entry.file_name().to_string_lossy().to_string();
        let target = match fs::read_link(entry.path()) {
            Ok(target) => target,
            Err(_) => continue,
        };
        if let Some(version) = target.file_name() {
            versions.insert(tool, version.to_string_lossy().to_string());
        }
    }

    Ok(versions)
}

pub fn retained_versions(vex_dir: &Path, cwd: &Path) -> Result<HashMap<(String, String), String>> {
    let mut retained = HashMap::new();

    for (tool, version) in read_current_versions(vex_dir)? {
        retained
            .entry((tool, version))
            .or_insert_with(|| "active".to_string());
    }

    let global_path = vex_dir.join("tool-versions");
    for (tool, version) in resolver::read_tool_versions_file(&global_path) {
        retained
            .entry((tool, version))
            .or_insert_with(|| "global default".to_string());
    }

    for (tool, version) in resolver::resolve_project_versions(cwd) {
        retained
            .entry((tool, version))
            .or_insert_with(|| "current project".to_string());
    }

    Ok(retained)
}
