mod env;

use crate::config;
use crate::error::{Result, VexError};
use crate::project::{self, LoadedProjectConfig};
use crate::resolver;
use crate::tools;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use env::{build_environment, push_path_entry, resolve_venv_dir};

#[derive(Debug, Clone)]
pub struct ActivationPlan {
    pub env: BTreeMap<String, String>,
    pub project: Option<LoadedProjectConfig>,
}

pub fn build_activation_plan(cwd: &Path) -> Result<ActivationPlan> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let toolchains_dir = config::toolchains_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    let project = project::load_nearest_project_config(cwd)?;
    let versions = resolver::resolve_versions(cwd);
    let venv_dir = resolve_venv_dir(cwd, project.as_ref())?;
    let path_entries = collect_path_entries(&toolchains_dir, &versions, venv_dir.as_deref())?;
    let env = build_environment(
        project.as_ref(),
        venv_dir.as_deref(),
        &vex_dir,
        &path_entries,
    );

    Ok(ActivationPlan { env, project })
}

fn collect_path_entries(
    toolchains_dir: &Path,
    versions: &std::collections::HashMap<String, String>,
    venv_dir: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    let mut path_entries = Vec::new();
    let mut path_seen = BTreeSet::new();

    if let Some(venv_dir) = venv_dir {
        push_path_entry(&mut path_entries, &mut path_seen, venv_dir.join("bin"));
    }

    let mut tool_names = versions.keys().cloned().collect::<Vec<_>>();
    tool_names.sort();

    for tool_name in tool_names {
        let version = versions.get(&tool_name).cloned().ok_or_else(|| {
            VexError::Parse(format!("Missing resolved version for {}", tool_name))
        })?;
        append_tool_bin_paths(
            &mut path_entries,
            &mut path_seen,
            toolchains_dir,
            &tool_name,
            &version,
        )?;
    }

    Ok(path_entries)
}

fn append_tool_bin_paths(
    path_entries: &mut Vec<PathBuf>,
    path_seen: &mut BTreeSet<PathBuf>,
    toolchains_dir: &Path,
    tool_name: &str,
    version: &str,
) -> Result<()> {
    let tool = tools::get_tool(tool_name)?;
    let install_dir = toolchains_dir.join(tool_name).join(version);
    if !install_dir.exists() {
        return Err(VexError::VersionNotFound {
            tool: tool_name.to_string(),
            version: version.to_string(),
            suggestions: String::new(),
        });
    }

    let mut local_seen = BTreeSet::new();
    for (_, subpath) in tool.bin_paths() {
        let bin_dir = install_dir.join(subpath);
        if bin_dir.exists() && local_seen.insert(bin_dir.clone()) {
            push_path_entry(path_entries, path_seen, bin_dir);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
