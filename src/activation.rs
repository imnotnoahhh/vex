mod env;

use crate::config;
use crate::error::{Result, VexError};
use crate::project::{self, LoadedProjectConfig};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use env::{
    build_set_env, build_unset_env, collect_exec_path_entries, collect_shared_path_entries,
    merged_path, original_path, resolve_active_versions, resolve_venv_dir,
};

#[derive(Debug, Clone)]
pub struct ActivationPlan {
    pub set_env: BTreeMap<String, String>,
    pub unset_env: Vec<String>,
    pub shared_path_entries: Vec<PathBuf>,
    pub exec_path_entries: Vec<PathBuf>,
    pub project: Option<LoadedProjectConfig>,
}

pub fn build_activation_plan(cwd: &Path) -> Result<ActivationPlan> {
    let settings = config::load_effective_settings(cwd)?;
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let toolchains_dir = config::toolchains_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    let project = project::load_nearest_project_config(cwd)?;
    let versions = resolve_active_versions(cwd, &vex_dir)?;
    let venv_dir = resolve_venv_dir(cwd, project.as_ref())?;
    let shared_path_entries = collect_shared_path_entries(
        cwd,
        &vex_dir,
        &toolchains_dir,
        &versions,
        venv_dir.as_deref(),
        settings.behavior.capture_user_state,
    )?;
    let exec_path_entries = collect_exec_path_entries(&toolchains_dir, &versions)?;
    let set_env = build_set_env(
        project.as_ref(),
        &vex_dir,
        &toolchains_dir,
        &versions,
        venv_dir.as_deref(),
        settings.behavior.capture_user_state,
    )?;
    let unset_env = build_unset_env(
        &versions,
        venv_dir.is_some(),
        settings.behavior.capture_user_state,
    )?;

    Ok(ActivationPlan {
        set_env,
        unset_env,
        shared_path_entries,
        exec_path_entries,
        project,
    })
}

pub fn exec_path(plan: &ActivationPlan) -> String {
    merged_path(
        plan.shared_path_entries
            .iter()
            .chain(plan.exec_path_entries.iter())
            .cloned()
            .collect::<Vec<_>>()
            .as_slice(),
        &original_path(),
    )
}

pub fn shell_path(plan: &ActivationPlan) -> Result<String> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let mut entries = plan.shared_path_entries.clone();
    entries.insert(usize::from(!entries.is_empty()), vex_dir.join("bin"));
    Ok(merged_path(&entries, &original_path()))
}

#[cfg(test)]
mod tests;
