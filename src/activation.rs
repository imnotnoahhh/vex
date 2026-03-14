use crate::config;
use crate::error::{Result, VexError};
use crate::project::{self, LoadedProjectConfig};
use crate::resolver;
use crate::tools;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ActivationPlan {
    pub env: BTreeMap<String, String>,
    pub project: Option<LoadedProjectConfig>,
}

pub fn build_activation_plan(cwd: &Path) -> Result<ActivationPlan> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let project = project::load_nearest_project_config(cwd)?;
    let versions = resolver::resolve_versions(cwd);
    let mut path_entries = Vec::new();
    let mut path_seen = BTreeSet::new();

    if let Some(venv_dir) = resolve_venv_dir(cwd, project.as_ref())? {
        let venv_bin = venv_dir.join("bin");
        push_path_entry(&mut path_entries, &mut path_seen, venv_bin);
    }

    let mut tool_names = versions.keys().cloned().collect::<Vec<_>>();
    tool_names.sort();

    for tool_name in tool_names {
        let version = versions.get(&tool_name).cloned().ok_or_else(|| {
            VexError::Parse(format!("Missing resolved version for {}", tool_name))
        })?;
        let tool = tools::get_tool(&tool_name)?;
        let install_dir = vex_dir.join("toolchains").join(&tool_name).join(&version);
        if !install_dir.exists() {
            return Err(VexError::VersionNotFound {
                tool: tool_name,
                version,
                suggestions: String::new(),
            });
        }

        let mut local_seen = BTreeSet::new();
        for (_, subpath) in tool.bin_paths() {
            let bin_dir = install_dir.join(subpath);
            if bin_dir.exists() && local_seen.insert(bin_dir.clone()) {
                push_path_entry(&mut path_entries, &mut path_seen, bin_dir.clone());
            }
        }
    }

    let mut env = BTreeMap::new();
    if let Some(project) = &project {
        for (key, value) in &project.config.env {
            if !key.trim().is_empty() {
                env.insert(key.trim().to_string(), value.clone());
            }
        }
    }

    if let Some(venv_dir) = resolve_venv_dir(cwd, project.as_ref())? {
        env.insert("VIRTUAL_ENV".to_string(), venv_dir.display().to_string());
        env.insert("VIRTUAL_ENV_DISABLE_PROMPT".to_string(), "1".to_string());
    }

    if let Ok(existing_cargo_home) = std::env::var("CARGO_HOME") {
        env.insert("CARGO_HOME".to_string(), existing_cargo_home);
    } else {
        env.insert(
            "CARGO_HOME".to_string(),
            vex_dir.join("cargo").display().to_string(),
        );
    }

    let original_path = std::env::var("PATH").unwrap_or_default();
    let merged_path = path_entries
        .iter()
        .map(|path| path.display().to_string())
        .chain(std::iter::once(original_path))
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(":");
    env.insert("PATH".to_string(), merged_path);

    Ok(ActivationPlan { env, project })
}

fn resolve_venv_dir(cwd: &Path, project: Option<&LoadedProjectConfig>) -> Result<Option<PathBuf>> {
    let project_auto_activate = project
        .and_then(|config| config.config.behavior.auto_activate_venv)
        .unwrap_or(config::auto_activate_venv()?);
    if !project_auto_activate {
        return Ok(None);
    }

    let Some(venv_dir) = project::find_nearest_venv(cwd) else {
        return Ok(None);
    };
    if venv_dir.join("bin").exists() {
        Ok(Some(venv_dir))
    } else {
        Ok(None)
    }
}

fn push_path_entry(entries: &mut Vec<PathBuf>, seen: &mut BTreeSet<PathBuf>, entry: PathBuf) {
    if seen.insert(entry.clone()) {
        entries.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_activation_plan_uses_project_venv_and_toolchain_bins() {
        let home = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        let vex_dir = home.path().join(".vex");
        let toolchain_bin = vex_dir.join("toolchains/node/20.11.0/bin");
        fs::create_dir_all(&toolchain_bin).unwrap();
        fs::create_dir_all(project.path().join(".venv/bin")).unwrap();
        fs::write(project.path().join(".tool-versions"), "node 20.11.0\n").unwrap();

        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home.path());
        let plan = build_activation_plan(project.path()).unwrap();

        let path = plan.env.get("PATH").cloned().unwrap_or_default();
        assert!(path.starts_with(project.path().join(".venv/bin").to_string_lossy().as_ref()));
        assert!(path.contains(toolchain_bin.to_string_lossy().as_ref()));
        let expected_venv = project.path().join(".venv").display().to_string();
        assert_eq!(plan.env.get("VIRTUAL_ENV").cloned(), Some(expected_venv));

        if let Some(value) = old_home {
            std::env::set_var("HOME", value);
        } else {
            std::env::remove_var("HOME");
        }
    }
}
