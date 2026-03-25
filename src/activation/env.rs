use crate::config;
use crate::error::Result;
use crate::project::{self, LoadedProjectConfig};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

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

    let Some(venv_dir) = project::find_nearest_venv(cwd) else {
        return Ok(None);
    };
    if venv_dir.join("bin").exists() {
        Ok(Some(venv_dir))
    } else {
        Ok(None)
    }
}

pub(super) fn push_path_entry(
    entries: &mut Vec<PathBuf>,
    seen: &mut BTreeSet<PathBuf>,
    entry: PathBuf,
) {
    if seen.insert(entry.clone()) {
        entries.push(entry);
    }
}

pub(super) fn build_environment(
    project: Option<&LoadedProjectConfig>,
    venv_dir: Option<&Path>,
    vex_dir: &Path,
    path_entries: &[PathBuf],
) -> BTreeMap<String, String> {
    let mut env = project_env(project);

    if let Some(venv_dir) = venv_dir {
        env.insert("VIRTUAL_ENV".to_string(), venv_dir.display().to_string());
        env.insert("VIRTUAL_ENV_DISABLE_PROMPT".to_string(), "1".to_string());
    }

    let cargo_home =
        std::env::var("CARGO_HOME").unwrap_or_else(|_| vex_dir.join("cargo").display().to_string());
    env.insert("CARGO_HOME".to_string(), cargo_home);
    env.insert("PATH".to_string(), merged_path(path_entries));
    env
}

fn project_env(project: Option<&LoadedProjectConfig>) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();

    if let Some(project) = project {
        for (key, value) in &project.config.env {
            let key = key.trim();
            if !key.is_empty() {
                env.insert(key.to_string(), value.clone());
            }
        }
    }

    env
}

fn merged_path(path_entries: &[PathBuf]) -> String {
    let original_path = std::env::var("PATH").unwrap_or_default();
    path_entries
        .iter()
        .map(|path| path.display().to_string())
        .chain(std::iter::once(original_path))
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(":")
}
