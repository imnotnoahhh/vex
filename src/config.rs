//! Unified configuration management for vex.
//!
//! This module provides:
//! - stable defaults for filesystem and networking behavior
//! - loading from `~/.vex/config.toml`
//! - environment variable overrides for CI and enterprise environments
//! - a single typed settings model the rest of the codebase can reuse

mod env;
mod layers;
mod model;
mod paths;

use crate::error::{Result, VexError};
use crate::project;
use crate::resolver;
use std::path::Path;
use std::time::Duration;

use env::apply_env_overrides;
use layers::{apply_file_config, apply_project_config, read_file_config};
use model::{
    BehaviorSettings, NetworkSettings, BIN_DIR, CACHE_DIR, CURRENT_DIR, MAX_CACHE_TTL,
    MIN_CACHE_TTL, TOOLCHAINS_DIR, VEX_DIR_NAME,
};
pub use model::{Settings, CHECKSUM_BUFFER_SIZE, DOWNLOAD_BUFFER_SIZE, MIN_FREE_SPACE_BYTES};
#[cfg(test)]
pub use model::{CONNECT_TIMEOUT, MAX_CONCURRENT_DOWNLOADS, READ_TIMEOUT, RETRY_BASE_DELAY};
pub use paths::{bin_dir, cache_dir, config_path, current_dir, toolchains_dir, vex_home};

pub fn load_settings() -> Result<Settings> {
    load_settings_internal(config_path().as_deref(), true)
}

pub fn load_effective_settings(start_dir: &Path) -> Result<Settings> {
    let mut settings = load_settings_internal(config_path().as_deref(), false)?;

    if let Some(project_config) = project::load_nearest_project_config(start_dir)? {
        apply_project_config(&mut settings, &project_config.config);
    }

    apply_env_overrides(&mut settings);
    Ok(settings)
}

pub fn load_effective_settings_for_current_dir() -> Result<Settings> {
    load_effective_settings(&resolver::current_dir())
}

pub fn load_settings_from_file(path: &Path) -> Result<Settings> {
    load_settings_internal(Some(path), false)
}

pub fn download_retries() -> Result<u32> {
    Ok(load_settings()?.network.download_retries)
}

pub fn cache_ttl() -> Result<Duration> {
    Ok(load_settings()?.cache_ttl)
}

pub fn auto_switch() -> Result<bool> {
    Ok(load_settings()?.behavior.auto_switch)
}

pub fn auto_activate_venv() -> Result<bool> {
    Ok(load_settings()?.behavior.auto_activate_venv)
}

pub fn default_shell() -> Result<Option<String>> {
    Ok(load_settings()?.behavior.default_shell)
}

pub fn non_interactive() -> Result<bool> {
    Ok(load_settings()?.behavior.non_interactive)
}

pub fn rewrite_download_url_with_settings(
    settings: &Settings,
    tool_name: &str,
    url: &str,
) -> Result<String> {
    let tool_name = tool_name.to_ascii_lowercase();
    let Some(mirror_base) = settings.mirrors.get(&tool_name) else {
        return Ok(url.to_string());
    };

    let original = reqwest::Url::parse(url).map_err(|err| {
        VexError::Config(format!(
            "Invalid upstream download URL for {}: {}",
            tool_name, err
        ))
    })?;
    let mut mirror = reqwest::Url::parse(mirror_base).map_err(|err| {
        VexError::Config(format!("Invalid mirror URL for {}: {}", tool_name, err))
    })?;

    let original_path = original.path().trim_start_matches('/');
    let mirror_prefix = mirror.path().trim_end_matches('/');
    let rewritten_path = if mirror_prefix.is_empty() || mirror_prefix == "/" {
        format!("/{}", original_path)
    } else {
        format!("{}/{}", mirror_prefix, original_path)
    };

    mirror.set_path(&rewritten_path);
    mirror.set_query(original.query());
    Ok(mirror.to_string())
}

fn load_settings_internal(path: Option<&Path>, include_env: bool) -> Result<Settings> {
    let mut settings = Settings::default();

    if let Some(path) = path {
        if let Some(file_config) = read_file_config(path)? {
            apply_file_config(&mut settings, file_config);
        }
    }

    if include_env {
        apply_env_overrides(&mut settings);
    }

    Ok(settings)
}

#[cfg(test)]
mod tests;
