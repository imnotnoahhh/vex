use crate::requested_versions;
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

    if !requested_versions::version_matches_request(version_str, project_version) {
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
        Some(global_version)
            if requested_versions::version_matches_request(version_str, global_version) =>
        {
            (
                "Global default".to_string(),
                Some(global_path.display().to_string()),
            )
        }
        _ => ("Manual activation".to_string(), None),
    }
}
