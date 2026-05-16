use super::ShellProjectEnv;
use crate::project::LoadedProjectConfig;
use std::collections::BTreeMap;

const UNSAFE_SHELL_ENV_KEYS: &[&str] = &[
    "BASH_ENV",
    "ENV",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_SSH_COMMAND",
    "NODE_OPTIONS",
    "PYTHONHOME",
    "PYTHONPATH",
    "PYTHONSTARTUP",
    "RUBYOPT",
];

pub(in crate::activation) fn project_env_for_process(
    project: Option<&LoadedProjectConfig>,
) -> BTreeMap<String, String> {
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

pub(in crate::activation) fn project_env_for_shell(project: Option<&LoadedProjectConfig>) -> ShellProjectEnv {
    let mut env = BTreeMap::new();
    let mut blocked_keys = Vec::new();

    if let Some(project) = project {
        for (key, value) in &project.config.env {
            let key = key.trim();
            if key.is_empty() {
                continue;
            }

            if is_unsafe_shell_env_key(key) {
                blocked_keys.push(key.to_string());
            } else {
                env.insert(key.to_string(), value.clone());
            }
        }
    }

    ShellProjectEnv {
        values: env,
        blocked_keys,
    }
}

fn is_unsafe_shell_env_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    UNSAFE_SHELL_ENV_KEYS.contains(&upper.as_str())
        || upper.starts_with("DYLD_")
        || upper.starts_with("LD_")
}
