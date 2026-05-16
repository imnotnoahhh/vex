mod env;

use crate::config;
use crate::error::{Result, VexError};
use crate::project::{self, LoadedProjectConfig};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use env::{
    build_set_env, build_unset_env, collect_exec_path_entries, collect_shared_path_entries,
    current_shell_context, load_previous_shell_context, load_previous_shell_project_env_keys,
    merged_path, original_path, project_env_for_process, project_env_for_shell,
    resolve_active_versions, resolve_venv_dir, store_shell_state,
};

#[derive(Debug, Clone)]
pub struct ActivationPlan {
    pub set_env: BTreeMap<String, String>,
    pub unset_env: Vec<String>,
    pub shared_path_entries: Vec<PathBuf>,
    pub exec_path_entries: Vec<PathBuf>,
    pub project: Option<LoadedProjectConfig>,
    pub warnings: Vec<String>,
}

pub fn build_activation_plan(cwd: &Path) -> Result<ActivationPlan> {
    build_activation_plan_with_mode(cwd, ActivationMode::Process)
}

pub fn build_shell_activation_plan(cwd: &Path) -> Result<ActivationPlan> {
    build_activation_plan_with_mode(cwd, ActivationMode::Shell)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActivationMode {
    Process,
    Shell,
}

fn build_activation_plan_with_mode(cwd: &Path, mode: ActivationMode) -> Result<ActivationPlan> {
    let settings = config::load_effective_settings(cwd)?;
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let toolchains_dir = config::toolchains_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    let project = project::load_nearest_project_config(cwd)?;
    let versions = resolve_active_versions(cwd, &vex_dir)?;
    let venv_dir = resolve_venv_dir(cwd, project.as_ref())?;
    let mut warnings = Vec::new();
    let project_env = match mode {
        ActivationMode::Process => project_env_for_process(project.as_ref()),
        ActivationMode::Shell => {
            let resolved = project_env_for_shell(project.as_ref());
            if !resolved.blocked_keys.is_empty() {
                warnings.push(format!(
                    "vex: skipped unsafe project env keys from .vex.toml: {}",
                    resolved.blocked_keys.join(", ")
                ));
            }
            resolved.values
        }
    };
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
        project_env,
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
    let mut unset_env = unset_env;

    if mode == ActivationMode::Shell {
        let current_project_env_keys = set_env
            .keys()
            .filter(|key| {
                project
                    .as_ref()
                    .is_some_and(|project| project.config.env.contains_key(key.as_str()))
            })
            .cloned()
            .collect::<Vec<_>>();
        let previous_project_env_keys = match load_previous_shell_project_env_keys(&vex_dir) {
            Ok(keys) => keys,
            Err(error) => {
                warnings.push(format!(
                    "vex: could not read previous project env state: {}",
                    error
                ));
                Vec::new()
            }
        };
        for key in previous_project_env_keys {
            if !current_project_env_keys.contains(&key) && !unset_env.contains(&key) {
                unset_env.push(key);
            }
        }

        let previous_context = match load_previous_shell_context(&vex_dir) {
            Ok(context) => context,
            Err(error) => {
                warnings.push(format!(
                    "vex: could not read previous activation state: {}",
                    error
                ));
                None
            }
        };
        let context = current_shell_context(
            project.as_ref().map(|project| project.root.as_path()),
            venv_dir.as_deref(),
            &current_project_env_keys,
        );
        if previous_context.as_deref() != Some(context.as_str()) {
            if let Some(venv_dir) = &venv_dir {
                warnings.push(format!(
                    "vex: auto-activated project .venv at {}",
                    venv_dir.display()
                ));
            }
            if !current_project_env_keys.is_empty() {
                warnings.push(format!(
                    "vex: applied project env from .vex.toml: {}",
                    current_project_env_keys.join(", ")
                ));
            }
        }
        if let Err(error) = store_shell_state(&vex_dir, &current_project_env_keys, &context) {
            warnings.push(format!("vex: could not save activation state: {}", error));
        }
    }

    Ok(ActivationPlan {
        set_env,
        unset_env,
        shared_path_entries,
        exec_path_entries,
        project,
        warnings,
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
