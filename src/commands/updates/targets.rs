mod installed;
mod scope;

use super::ManagedSource;
use crate::config;
use crate::error::{Result, VexError};
use crate::resolver;
use crate::tools;
use crate::version_state;
pub(super) use crate::versioning::normalize_version;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(super) struct ManagedTarget {
    pub tool: String,
    pub version: String,
    pub source: ManagedSource,
    pub source_path: Option<PathBuf>,
}

pub(super) fn collect_targets(tool_filter: Option<&str>) -> Result<(String, Vec<ManagedTarget>)> {
    let cwd = resolver::current_dir();
    let project_versions = resolver::resolve_project_versions(&cwd);
    let vex_home = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let global_path = vex_home.join("tool-versions");
    let global_versions = resolver::read_tool_versions_file(&global_path);
    let current_versions = version_state::read_current_versions(&vex_home)?;

    if let Some(tool_name) = tool_filter {
        let _ = tools::get_tool(tool_name)?;
        let explicit = scope::collect_explicit_target(
            &cwd,
            tool_name,
            &project_versions,
            &global_path,
            &global_versions,
            &current_versions,
            installed::latest_installed_version(tool_name)?,
        );
        return Ok((
            "explicit".to_string(),
            explicit.into_iter().collect::<Vec<_>>(),
        ));
    }

    if !project_versions.is_empty() {
        return Ok((
            "project".to_string(),
            scope::project_targets(&cwd, &project_versions),
        ));
    }

    if !global_versions.is_empty() {
        return Ok((
            "global".to_string(),
            scope::global_targets(&global_path, &global_versions),
        ));
    }

    Ok((
        "active".to_string(),
        scope::active_targets(current_versions),
    ))
}
