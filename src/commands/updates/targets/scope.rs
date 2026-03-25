use super::{ManagedSource, ManagedTarget};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn collect_explicit_target(
    cwd: &Path,
    tool_name: &str,
    project_versions: &HashMap<String, String>,
    global_path: &Path,
    global_versions: &HashMap<String, String>,
    current_versions: &HashMap<String, String>,
    latest_installed_version: Option<String>,
) -> Option<ManagedTarget> {
    if let Some(version) = project_versions.get(tool_name) {
        return Some(ManagedTarget {
            tool: tool_name.to_string(),
            version: version.clone(),
            source: ManagedSource::Project,
            source_path: crate::resolver::find_project_source(cwd, tool_name),
        });
    }

    if let Some(version) = global_versions.get(tool_name) {
        return Some(ManagedTarget {
            tool: tool_name.to_string(),
            version: version.clone(),
            source: ManagedSource::Global,
            source_path: Some(global_path.to_path_buf()),
        });
    }

    if let Some(version) = current_versions.get(tool_name) {
        return Some(ManagedTarget {
            tool: tool_name.to_string(),
            version: version.clone(),
            source: ManagedSource::Active,
            source_path: None,
        });
    }

    latest_installed_version.map(|version| ManagedTarget {
        tool: tool_name.to_string(),
        version,
        source: ManagedSource::Installed,
        source_path: None,
    })
}

pub(super) fn project_targets(
    cwd: &Path,
    project_versions: &HashMap<String, String>,
) -> Vec<ManagedTarget> {
    let mut targets = project_versions
        .iter()
        .map(|(tool, version)| ManagedTarget {
            tool: tool.clone(),
            version: version.clone(),
            source: ManagedSource::Project,
            source_path: crate::resolver::find_project_source(cwd, tool),
        })
        .collect::<Vec<_>>();
    sort_targets(&mut targets);
    targets
}

pub(super) fn global_targets(
    global_path: &Path,
    global_versions: &HashMap<String, String>,
) -> Vec<ManagedTarget> {
    let mut targets = global_versions
        .iter()
        .map(|(tool, version)| ManagedTarget {
            tool: tool.clone(),
            version: version.clone(),
            source: ManagedSource::Global,
            source_path: Some(PathBuf::from(global_path)),
        })
        .collect::<Vec<_>>();
    sort_targets(&mut targets);
    targets
}

pub(super) fn active_targets(current_versions: HashMap<String, String>) -> Vec<ManagedTarget> {
    let mut targets = current_versions
        .into_iter()
        .map(|(tool, version)| ManagedTarget {
            tool,
            version,
            source: ManagedSource::Active,
            source_path: None,
        })
        .collect::<Vec<_>>();
    sort_targets(&mut targets);
    targets
}

fn sort_targets(targets: &mut [ManagedTarget]) {
    targets.sort_by(|left, right| left.tool.cmp(&right.tool));
}
