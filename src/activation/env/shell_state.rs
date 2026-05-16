use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

const SHELL_PROJECT_ENV_KEYS_FILE: &str = "project-env-keys";
const SHELL_ACTIVATION_CONTEXT_FILE: &str = "activation-context";

pub(in crate::activation) fn load_previous_shell_project_env_keys(vex_dir: &Path) -> Result<Vec<String>> {
    let path = shell_state_path(vex_dir, SHELL_PROJECT_ENV_KEYS_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let keys = fs::read_to_string(path)?
        .lines()
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    Ok(keys)
}

pub(in crate::activation) fn load_previous_shell_context(vex_dir: &Path) -> Result<Option<String>> {
    let path = shell_state_path(vex_dir, SHELL_ACTIVATION_CONTEXT_FILE);
    if !path.exists() {
        return Ok(None);
    }

    Ok(Some(fs::read_to_string(path)?))
}

pub(in crate::activation) fn store_shell_state(
    vex_dir: &Path,
    project_env_keys: &[String],
    context: &str,
) -> Result<()> {
    let state_dir = vex_dir.join("state");
    fs::create_dir_all(&state_dir)?;

    let mut keys = project_env_keys.to_vec();
    keys.sort();
    fs::write(
        state_dir.join(SHELL_PROJECT_ENV_KEYS_FILE),
        if keys.is_empty() {
            String::new()
        } else {
            format!("{}\n", keys.join("\n"))
        },
    )?;
    fs::write(state_dir.join(SHELL_ACTIVATION_CONTEXT_FILE), context)?;
    Ok(())
}

pub(in crate::activation) fn current_shell_context(
    project_root: Option<&Path>,
    venv_dir: Option<&Path>,
    project_env_keys: &[String],
) -> String {
    let mut keys = project_env_keys.to_vec();
    keys.sort();
    format!(
        "project={}\nvenv={}\nenv={}\n",
        project_root
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        venv_dir
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        keys.join(",")
    )
}

fn shell_state_path(vex_dir: &Path, filename: &str) -> PathBuf {
    vex_dir.join("state").join(filename)
}
