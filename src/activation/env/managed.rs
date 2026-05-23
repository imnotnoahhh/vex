use super::{checked_install_dir, ALWAYS_MANAGED_ENV_KEYS, ALWAYS_UNSET_ENV_KEYS, SUPPORTED_TOOLS};
use crate::error::Result;
use crate::tools;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(in crate::activation) fn build_set_env(
    mut env: BTreeMap<String, String>,
    vex_dir: &Path,
    toolchains_dir: &Path,
    versions: &BTreeMap<String, String>,
    venv_dir: Option<&Path>,
    capture_user_state: bool,
) -> Result<BTreeMap<String, String>> {
    for (tool_name, version) in versions {
        let tool = tools::get_tool(tool_name)?;
        let install_dir = checked_install_dir(toolchains_dir, tool_name, version)?;
        let tool_env = tool.managed_environment(vex_dir, Some(&install_dir));
        for (key, value) in filter_managed_env(tool_env.managed_env, capture_user_state) {
            env.insert(key, value);
        }
    }

    if let Some(venv_dir) = venv_dir {
        env.insert("VIRTUAL_ENV".to_string(), venv_dir.display().to_string());
        env.insert("VIRTUAL_ENV_DISABLE_PROMPT".to_string(), "1".to_string());
    }

    Ok(env)
}

pub(in crate::activation) fn build_unset_env(
    versions: &BTreeMap<String, String>,
    has_venv: bool,
    capture_user_state: bool,
) -> Result<Vec<String>> {
    let mut active_keys = BTreeSet::new();
    for tool_name in versions.keys() {
        let tool = tools::get_tool(tool_name)?;
        for key in tool.managed_env_keys() {
            if (capture_user_state || ALWAYS_MANAGED_ENV_KEYS.contains(&key))
                && !ALWAYS_UNSET_ENV_KEYS.contains(&key)
            {
                active_keys.insert(key.to_string());
            }
        }
    }

    if has_venv {
        active_keys.insert("VIRTUAL_ENV".to_string());
        active_keys.insert("VIRTUAL_ENV_DISABLE_PROMPT".to_string());
    }

    let mut managed_keys = BTreeSet::from([
        "VIRTUAL_ENV".to_string(),
        "VIRTUAL_ENV_DISABLE_PROMPT".to_string(),
    ]);

    for tool_name in SUPPORTED_TOOLS {
        let tool = tools::get_tool(tool_name)?;
        for key in tool.managed_env_keys() {
            if capture_user_state || ALWAYS_MANAGED_ENV_KEYS.contains(&key) {
                managed_keys.insert(key.to_string());
            }
        }
    }

    Ok(managed_keys
        .difference(&active_keys)
        .cloned()
        .collect::<Vec<_>>())
}

fn filter_managed_env(
    managed_env: BTreeMap<String, String>,
    capture_user_state: bool,
) -> BTreeMap<String, String> {
    managed_env
        .into_iter()
        .filter(|(key, _)| {
            (capture_user_state || ALWAYS_MANAGED_ENV_KEYS.contains(&key.as_str()))
                && !ALWAYS_UNSET_ENV_KEYS.contains(&key.as_str())
        })
        .collect()
}
