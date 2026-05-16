mod managed;
mod paths;
mod project_env;
mod shell_state;

use crate::config;
use crate::error::{Result, VexError};
use crate::project::{self as project_mod, LoadedProjectConfig};
use crate::requested_versions;
use crate::resolver;
use crate::version_state;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) use managed::{build_set_env, build_unset_env};
pub(super) use paths::{
    collect_exec_path_entries, collect_shared_path_entries, merged_path, original_path,
};
pub(super) use project_env::{project_env_for_process, project_env_for_shell};
pub(super) use shell_state::{
    current_shell_context, load_previous_shell_context, load_previous_shell_project_env_keys,
    store_shell_state,
};

pub(in crate::activation) const SUPPORTED_TOOLS: &[&str] =
    &["go", "java", "node", "python", "rust"];
pub(in crate::activation) const ALWAYS_MANAGED_ENV_KEYS: &[&str] = &["GOROOT", "JAVA_HOME"];

#[derive(Debug, Clone)]
pub(in crate::activation) struct ShellProjectEnv {
    pub values: BTreeMap<String, String>,
    pub blocked_keys: Vec<String>,
}

pub(super) fn resolve_venv_dir(
    cwd: &Path,
    project: Option<&LoadedProjectConfig>,
) -> Result<Option<PathBuf>> {
    let project_auto_activate = project
        .and_then(|config| config.config.behavior.auto_activate_venv)
        .unwrap_or(config::auto_activate_venv()?);
    if !project_auto_activate {
        return Ok(None);
    }

    let Some(venv_dir) = project_mod::find_nearest_venv(cwd) else {
        return Ok(None);
    };
    if venv_dir.join("bin").exists() {
        Ok(Some(venv_dir))
    } else {
        Ok(None)
    }
}

pub(super) fn resolve_active_versions(
    cwd: &Path,
    vex_dir: &Path,
) -> Result<BTreeMap<String, String>> {
    let requested = resolver::resolve_versions(cwd);
    if requested.is_empty() {
        return Ok(version_state::read_current_versions(vex_dir)?
            .into_iter()
            .collect());
    }

    requested
        .into_iter()
        .map(|(tool, requested)| {
            let resolved =
                requested_versions::resolve_installed_version(vex_dir, &tool, &requested)?
                    .unwrap_or(requested);
            Ok((tool, resolved))
        })
        .collect()
}

pub(in crate::activation) fn checked_install_dir(
    toolchains_dir: &Path,
    tool_name: &str,
    version: &str,
) -> Result<PathBuf> {
    let install_dir = toolchains_dir.join(tool_name).join(version);
    if install_dir.exists() {
        Ok(install_dir)
    } else {
        Err(VexError::VersionNotFound {
            tool: tool_name.to_string(),
            version: version.to_string(),
            suggestions: String::new(),
        })
    }
}

pub(in crate::activation) fn push_path_entry(
    entries: &mut Vec<PathBuf>,
    seen: &mut BTreeSet<PathBuf>,
    entry: PathBuf,
) {
    if seen.insert(entry.clone()) {
        entries.push(entry);
    }
}
