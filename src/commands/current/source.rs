use crate::resolver;
use std::collections::HashMap;
use std::path::Path;

pub(super) fn resolve_source(
    pwd: &Path,
    tool_name: &str,
    version_str: &str,
    versions: &HashMap<String, String>,
    global_path: &Path,
    global_versions: &HashMap<String, String>,
) -> (String, Option<String>) {
    let Some(project_version) = versions.get(tool_name) else {
        return global_or_manual(tool_name, version_str, global_path, global_versions);
    };

    if project_version != version_str {
        return global_or_manual(tool_name, version_str, global_path, global_versions);
    }

    resolver::find_project_source(pwd, tool_name)
        .map(|source_path| {
            (
                "Project override".to_string(),
                Some(source_path.display().to_string()),
            )
        })
        .unwrap_or_else(|| global_or_manual(tool_name, version_str, global_path, global_versions))
}

fn global_or_manual(
    tool_name: &str,
    version_str: &str,
    global_path: &Path,
    global_versions: &HashMap<String, String>,
) -> (String, Option<String>) {
    match global_versions.get(tool_name) {
        Some(global_version) if global_version == version_str => (
            "Global default".to_string(),
            Some(global_path.display().to_string()),
        ),
        _ => ("Manual activation".to_string(), None),
    }
}
