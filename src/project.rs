mod discovery;

use crate::error::{Result, VexError};
pub use discovery::{find_nearest_node_modules_bin, find_nearest_project_file, find_nearest_venv};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

pub fn load_nearest_project_config(start_dir: &Path) -> Result<Option<LoadedProjectConfig>> {
    let Some(path) = find_nearest_project_file(start_dir) else {
        return Ok(None);
    };

    let content = fs::read_to_string(&path)?;
    let config: ProjectConfig = toml::from_str(&content)
        .map_err(|err| VexError::Config(format!("Failed to parse {}: {}", path.display(), err)))?;
    validate_project_config(&path, &config)?;

    Ok(Some(LoadedProjectConfig {
        root: path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| start_dir.to_path_buf()),
        path,
        config,
    }))
}

fn validate_project_config(path: &Path, config: &ProjectConfig) -> Result<()> {
    for key in config.env.keys() {
        let key = key.trim();
        if !is_valid_env_key(key) {
            return Err(VexError::Config(format!(
                "Invalid environment variable name '{}' in {}. Names must match [A-Za-z_][A-Za-z0-9_]*.",
                key.escape_debug(),
                path.display()
            )));
        }
    }

    Ok(())
}

fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    matches!(first, 'A'..='Z' | 'a'..='z' | '_')
        && chars.all(|ch| matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_'))
}

#[cfg(test)]
mod tests;
