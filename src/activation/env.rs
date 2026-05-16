use crate::config;
use crate::error::{Result, VexError};
use crate::project::{self, LoadedProjectConfig};
use crate::requested_versions;
use crate::resolver;
use crate::tools;
use crate::version_state;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const SUPPORTED_TOOLS: &[&str] = &["go", "java", "node", "python", "rust"];
const ALWAYS_MANAGED_ENV_KEYS: &[&str] = &["GOROOT", "JAVA_HOME"];
const SHELL_PROJECT_ENV_KEYS_FILE: &str = "project-env-keys";
const SHELL_ACTIVATION_CONTEXT_FILE: &str = "activation-context";
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

#[derive(Debug, Clone)]
pub(super) struct ShellProjectEnv {
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

    let Some(venv_dir) = project::find_nearest_venv(cwd) else {
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

pub(super) fn collect_shared_path_entries(
    cwd: &Path,
    vex_dir: &Path,
    toolchains_dir: &Path,
    versions: &BTreeMap<String, String>,
    venv_dir: Option<&Path>,
    capture_user_state: bool,
) -> Result<Vec<PathBuf>> {
    let mut path_entries = Vec::new();
    let mut path_seen = BTreeSet::new();

    if let Some(venv_dir) = venv_dir {
        push_path_entry(&mut path_entries, &mut path_seen, venv_dir.join("bin"));
    }

    if versions.contains_key("node") {
        if let Some(node_modules_bin) = project::find_nearest_node_modules_bin(cwd) {
            push_path_entry(&mut path_entries, &mut path_seen, node_modules_bin);
        }
    }

    if capture_user_state {
        for (tool_name, version) in versions {
            if tool_name == "python" && venv_dir.is_some() {
                continue;
            }

            let tool = tools::get_tool(tool_name)?;
            let install_dir = checked_install_dir(toolchains_dir, tool_name, version)?;
            let environment = tool.managed_environment(vex_dir, Some(&install_dir));
            for path in environment.managed_user_bin_dirs {
                push_path_entry(&mut path_entries, &mut path_seen, PathBuf::from(path));
            }
        }
    }

    Ok(path_entries)
}

pub(super) fn collect_exec_path_entries(
    toolchains_dir: &Path,
    versions: &BTreeMap<String, String>,
) -> Result<Vec<PathBuf>> {
    let mut path_entries = Vec::new();
    let mut path_seen = BTreeSet::new();

    for (tool_name, version) in versions {
        append_tool_bin_paths(
            &mut path_entries,
            &mut path_seen,
            toolchains_dir,
            tool_name,
            version,
        )?;
    }

    Ok(path_entries)
}

pub(super) fn build_set_env(
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

pub(super) fn build_unset_env(
    versions: &BTreeMap<String, String>,
    has_venv: bool,
    capture_user_state: bool,
) -> Result<Vec<String>> {
    let mut active_keys = BTreeSet::new();
    for tool_name in versions.keys() {
        let tool = tools::get_tool(tool_name)?;
        for key in tool.managed_env_keys() {
            if capture_user_state || ALWAYS_MANAGED_ENV_KEYS.contains(&key) {
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

pub(super) fn merged_path(path_entries: &[PathBuf], original_path: &str) -> String {
    let mut merged = Vec::new();
    let mut seen = BTreeSet::new();

    for entry in path_entries {
        let segment = entry.display().to_string();
        if !segment.is_empty() && seen.insert(segment.clone()) {
            merged.push(segment);
        }
    }

    for segment in original_path
        .split(':')
        .filter(|segment| !segment.is_empty())
    {
        let segment = segment.to_string();
        if seen.insert(segment.clone()) {
            merged.push(segment);
        }
    }

    merged.join(":")
}

pub(super) fn original_path() -> String {
    std::env::var("VEX_ORIGINAL_PATH")
        .or_else(|_| std::env::var("PATH"))
        .unwrap_or_default()
}

pub(super) fn project_env_for_process(
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

pub(super) fn project_env_for_shell(project: Option<&LoadedProjectConfig>) -> ShellProjectEnv {
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

pub(super) fn load_previous_shell_project_env_keys(vex_dir: &Path) -> Result<Vec<String>> {
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

pub(super) fn load_previous_shell_context(vex_dir: &Path) -> Result<Option<String>> {
    let path = shell_state_path(vex_dir, SHELL_ACTIVATION_CONTEXT_FILE);
    if !path.exists() {
        return Ok(None);
    }

    Ok(Some(fs::read_to_string(path)?))
}

pub(super) fn store_shell_state(
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

pub(super) fn current_shell_context(
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

fn checked_install_dir(toolchains_dir: &Path, tool_name: &str, version: &str) -> Result<PathBuf> {
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

fn append_tool_bin_paths(
    path_entries: &mut Vec<PathBuf>,
    path_seen: &mut BTreeSet<PathBuf>,
    toolchains_dir: &Path,
    tool_name: &str,
    version: &str,
) -> Result<()> {
    let tool = tools::get_tool(tool_name)?;
    let install_dir = checked_install_dir(toolchains_dir, tool_name, version)?;

    let mut local_seen = BTreeSet::new();
    for (_, subpath) in tool.bin_paths() {
        let bin_dir = install_dir.join(subpath);
        if bin_dir.exists() && local_seen.insert(bin_dir.clone()) {
            push_path_entry(path_entries, path_seen, bin_dir);
        }
    }

    Ok(())
}

fn filter_managed_env(
    managed_env: BTreeMap<String, String>,
    capture_user_state: bool,
) -> BTreeMap<String, String> {
    managed_env
        .into_iter()
        .filter(|(key, _)| capture_user_state || ALWAYS_MANAGED_ENV_KEYS.contains(&key.as_str()))
        .collect()
}

fn is_unsafe_shell_env_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    UNSAFE_SHELL_ENV_KEYS.contains(&upper.as_str())
        || upper.starts_with("DYLD_")
        || upper.starts_with("LD_")
}

fn shell_state_path(vex_dir: &Path, filename: &str) -> PathBuf {
    vex_dir.join("state").join(filename)
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
