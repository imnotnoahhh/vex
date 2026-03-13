use crate::error::{Result, VexError};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const PROJECT_CONFIG_FILE: &str = ".vex.toml";
const PROJECT_VENV_DIR: &str = ".venv";

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProjectBehaviorConfig {
    pub auto_switch: Option<bool>,
    pub auto_activate_venv: Option<bool>,
    pub default_shell: Option<String>,
    pub non_interactive: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProjectNetworkConfig {
    pub connect_timeout_secs: Option<u64>,
    pub read_timeout_secs: Option<u64>,
    pub download_retries: Option<u32>,
    pub retry_base_delay_secs: Option<u64>,
    pub max_concurrent_downloads: Option<usize>,
    pub max_http_redirects: Option<usize>,
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub behavior: ProjectBehaviorConfig,
    #[serde(default)]
    pub network: ProjectNetworkConfig,
    #[serde(default)]
    pub mirrors: HashMap<String, String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub commands: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct LoadedProjectConfig {
    pub root: PathBuf,
    pub path: PathBuf,
    pub config: ProjectConfig,
}

pub fn find_nearest_project_file(start_dir: &Path) -> Option<PathBuf> {
    find_in_ancestors(start_dir, PROJECT_CONFIG_FILE)
}

pub fn find_nearest_venv(start_dir: &Path) -> Option<PathBuf> {
    find_in_ancestors(start_dir, PROJECT_VENV_DIR)
}

pub fn load_nearest_project_config(start_dir: &Path) -> Result<Option<LoadedProjectConfig>> {
    let Some(path) = find_nearest_project_file(start_dir) else {
        return Ok(None);
    };

    let content = fs::read_to_string(&path)?;
    let config: ProjectConfig = toml::from_str(&content)
        .map_err(|err| VexError::Parse(format!("Failed to parse {}: {}", path.display(), err)))?;

    Ok(Some(LoadedProjectConfig {
        root: path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| start_dir.to_path_buf()),
        path,
        config,
    }))
}

fn find_in_ancestors(start_dir: &Path, file_name: &str) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(file_name);
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_nearest_project_config() {
        let temp = TempDir::new().unwrap();
        let project = temp.path().join("project");
        let nested = project.join("src/bin");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            project.join(".vex.toml"),
            r#"
[behavior]
auto_switch = false

[env]
RUST_LOG = "debug"

[commands]
test = "cargo test"
"#,
        )
        .unwrap();

        let loaded = load_nearest_project_config(&nested)
            .unwrap()
            .expect("project config should load");
        assert_eq!(loaded.root, project);
        assert!(!loaded.config.behavior.auto_switch.unwrap());
        assert_eq!(
            loaded.config.env.get("RUST_LOG").map(String::as_str),
            Some("debug")
        );
        assert_eq!(
            loaded.config.commands.get("test").map(String::as_str),
            Some("cargo test")
        );
    }

    #[test]
    fn test_find_nearest_venv() {
        let temp = TempDir::new().unwrap();
        let project = temp.path().join("project");
        let nested = project.join("nested/deeper");
        fs::create_dir_all(project.join(".venv")).unwrap();
        fs::create_dir_all(&nested).unwrap();

        let venv = find_nearest_venv(&nested).expect("venv should be found");
        assert_eq!(venv, project.join(".venv"));
    }
}
