//! Unified configuration management for vex.
//!
//! This module provides:
//! - stable defaults for filesystem and networking behavior
//! - loading from `~/.vex/config.toml`
//! - environment variable overrides for CI and enterprise environments
//! - a single typed settings model the rest of the codebase can reuse

use crate::error::{Result, VexError};
use crate::project;
use crate::resolver;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// HTTP connection timeout (30 seconds)
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// HTTP read timeout (5 minutes, suitable for large file downloads)
pub const READ_TIMEOUT: Duration = Duration::from_secs(300);

/// Download buffer size (64 KB)
pub const DOWNLOAD_BUFFER_SIZE: usize = 65536;

/// Checksum calculation buffer size (64 KB)
pub const CHECKSUM_BUFFER_SIZE: usize = 65536;

/// Maximum number of download retry attempts
pub const MAX_DOWNLOAD_RETRIES: u32 = 3;

/// Base delay for exponential backoff (1 second)
pub const RETRY_BASE_DELAY: Duration = Duration::from_secs(1);

/// Maximum concurrent downloads
pub const MAX_CONCURRENT_DOWNLOADS: usize = 3;

/// HTTP redirect limit
pub const MAX_HTTP_REDIRECTS: usize = 10;

/// Minimum free disk space before installation (1.5 GB)
pub const MIN_FREE_SPACE_BYTES: u64 = 1536 * 1024 * 1024;

/// Cache TTL (5 minutes)
pub const CACHE_TTL: Duration = Duration::from_secs(300);

/// Minimum cache TTL (1 minute)
pub const MIN_CACHE_TTL: Duration = Duration::from_secs(60);

/// Maximum cache TTL (1 hour)
pub const MAX_CACHE_TTL: Duration = Duration::from_secs(3600);

/// vex home directory name
pub const VEX_DIR_NAME: &str = ".vex";

/// Toolchains subdirectory name
pub const TOOLCHAINS_DIR: &str = "toolchains";

/// Current version symlink directory name
pub const CURRENT_DIR: &str = "current";

/// Binary symlinks directory name
pub const BIN_DIR: &str = "bin";

/// Cache directory name
pub const CACHE_DIR: &str = "cache";

#[derive(Debug, Clone, Deserialize, Default)]
struct FileConfig {
    cache_ttl_secs: Option<u64>,
    #[serde(default)]
    network: NetworkFileConfig,
    #[serde(default)]
    behavior: BehaviorFileConfig,
    #[serde(default)]
    mirrors: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct NetworkFileConfig {
    connect_timeout_secs: Option<u64>,
    read_timeout_secs: Option<u64>,
    download_retries: Option<u32>,
    retry_base_delay_secs: Option<u64>,
    max_concurrent_downloads: Option<usize>,
    max_http_redirects: Option<usize>,
    proxy: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct BehaviorFileConfig {
    auto_switch: Option<bool>,
    auto_activate_venv: Option<bool>,
    default_shell: Option<String>,
    non_interactive: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkSettings {
    pub connect_timeout: Duration,
    pub read_timeout: Duration,
    pub download_retries: u32,
    pub retry_base_delay: Duration,
    pub max_concurrent_downloads: usize,
    pub max_http_redirects: usize,
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorSettings {
    pub auto_switch: bool,
    pub auto_activate_venv: bool,
    pub default_shell: Option<String>,
    pub non_interactive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub cache_ttl: Duration,
    pub network: NetworkSettings,
    pub behavior: BehaviorSettings,
    pub mirrors: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cache_ttl: CACHE_TTL,
            network: NetworkSettings {
                connect_timeout: CONNECT_TIMEOUT,
                read_timeout: READ_TIMEOUT,
                download_retries: MAX_DOWNLOAD_RETRIES,
                retry_base_delay: RETRY_BASE_DELAY,
                max_concurrent_downloads: MAX_CONCURRENT_DOWNLOADS,
                max_http_redirects: MAX_HTTP_REDIRECTS,
                proxy: None,
            },
            behavior: BehaviorSettings {
                auto_switch: true,
                auto_activate_venv: true,
                default_shell: None,
                non_interactive: false,
            },
            mirrors: HashMap::new(),
        }
    }
}

/// Get vex home directory path.
///
/// Returns `~/.vex` or the path specified by `VEX_HOME`.
pub fn vex_home() -> Option<PathBuf> {
    std::env::var("VEX_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|path| path.join(VEX_DIR_NAME)))
}

/// Get toolchains directory path.
pub fn toolchains_dir() -> Option<PathBuf> {
    vex_home().map(|path| path.join(TOOLCHAINS_DIR))
}

/// Get current version symlink directory path.
pub fn current_dir() -> Option<PathBuf> {
    vex_home().map(|path| path.join(CURRENT_DIR))
}

/// Get binary symlinks directory path.
pub fn bin_dir() -> Option<PathBuf> {
    vex_home().map(|path| path.join(BIN_DIR))
}

/// Get cache directory path.
pub fn cache_dir() -> Option<PathBuf> {
    vex_home().map(|path| path.join(CACHE_DIR))
}

pub fn config_path() -> Option<PathBuf> {
    vex_home().map(|path| path.join("config.toml"))
}

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
        VexError::Parse(format!(
            "Invalid upstream download URL for {}: {}",
            tool_name, err
        ))
    })?;
    let mut mirror = reqwest::Url::parse(mirror_base)
        .map_err(|err| VexError::Parse(format!("Invalid mirror URL for {}: {}", tool_name, err)))?;

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

fn read_file_config(path: &Path) -> Result<Option<FileConfig>> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(VexError::Io(err)),
    };

    toml::from_str(&content)
        .map(Some)
        .map_err(|err| VexError::Parse(format!("Failed to parse {}: {}", path.display(), err)))
}

fn apply_file_config(settings: &mut Settings, file_config: FileConfig) {
    if let Some(cache_ttl_secs) = file_config.cache_ttl_secs {
        settings.cache_ttl = validated_cache_ttl(cache_ttl_secs);
    }

    if let Some(secs) = file_config.network.connect_timeout_secs {
        settings.network.connect_timeout = Duration::from_secs(secs.max(1));
    }
    if let Some(secs) = file_config.network.read_timeout_secs {
        settings.network.read_timeout = Duration::from_secs(secs.max(1));
    }
    if let Some(retries) = file_config.network.download_retries {
        settings.network.download_retries = retries.max(1);
    }
    if let Some(secs) = file_config.network.retry_base_delay_secs {
        settings.network.retry_base_delay = Duration::from_secs(secs.max(1));
    }
    if let Some(concurrency) = file_config.network.max_concurrent_downloads {
        settings.network.max_concurrent_downloads = concurrency.max(1);
    }
    if let Some(redirects) = file_config.network.max_http_redirects {
        settings.network.max_http_redirects = redirects.max(1);
    }
    if let Some(proxy) = file_config.network.proxy {
        settings.network.proxy = non_empty(proxy);
    }

    if let Some(auto_switch) = file_config.behavior.auto_switch {
        settings.behavior.auto_switch = auto_switch;
    }
    if let Some(auto_activate_venv) = file_config.behavior.auto_activate_venv {
        settings.behavior.auto_activate_venv = auto_activate_venv;
    }
    if let Some(default_shell) = file_config.behavior.default_shell {
        settings.behavior.default_shell = non_empty(default_shell);
    }
    if let Some(non_interactive) = file_config.behavior.non_interactive {
        settings.behavior.non_interactive = non_interactive;
    }

    for (tool, mirror) in file_config.mirrors {
        if let Some(mirror) = non_empty(mirror) {
            settings
                .mirrors
                .insert(tool.trim().to_ascii_lowercase(), mirror);
        }
    }
}

fn apply_project_config(settings: &mut Settings, project_config: &project::ProjectConfig) {
    if let Some(secs) = project_config.network.connect_timeout_secs {
        settings.network.connect_timeout = Duration::from_secs(secs.max(1));
    }
    if let Some(secs) = project_config.network.read_timeout_secs {
        settings.network.read_timeout = Duration::from_secs(secs.max(1));
    }
    if let Some(retries) = project_config.network.download_retries {
        settings.network.download_retries = retries.max(1);
    }
    if let Some(secs) = project_config.network.retry_base_delay_secs {
        settings.network.retry_base_delay = Duration::from_secs(secs.max(1));
    }
    if let Some(concurrency) = project_config.network.max_concurrent_downloads {
        settings.network.max_concurrent_downloads = concurrency.max(1);
    }
    if let Some(redirects) = project_config.network.max_http_redirects {
        settings.network.max_http_redirects = redirects.max(1);
    }
    if let Some(proxy) = project_config.network.proxy.clone() {
        settings.network.proxy = non_empty(proxy);
    }

    if let Some(auto_switch) = project_config.behavior.auto_switch {
        settings.behavior.auto_switch = auto_switch;
    }
    if let Some(auto_activate_venv) = project_config.behavior.auto_activate_venv {
        settings.behavior.auto_activate_venv = auto_activate_venv;
    }
    if let Some(default_shell) = project_config.behavior.default_shell.clone() {
        settings.behavior.default_shell = non_empty(default_shell);
    }
    if let Some(non_interactive) = project_config.behavior.non_interactive {
        settings.behavior.non_interactive = non_interactive;
    }

    for (tool, mirror) in &project_config.mirrors {
        if let Some(mirror) = non_empty(mirror.clone()) {
            settings
                .mirrors
                .insert(tool.trim().to_ascii_lowercase(), mirror);
        }
    }
}

fn apply_env_overrides(settings: &mut Settings) {
    if let Some(value) = env_u64("VEX_CACHE_TTL_SECS") {
        settings.cache_ttl = validated_cache_ttl(value);
    }
    if let Some(value) = env_u64("VEX_CONNECT_TIMEOUT_SECS") {
        settings.network.connect_timeout = Duration::from_secs(value.max(1));
    }
    if let Some(value) = env_u64("VEX_READ_TIMEOUT_SECS") {
        settings.network.read_timeout = Duration::from_secs(value.max(1));
    }
    if let Some(value) = env_u32("VEX_DOWNLOAD_RETRIES") {
        settings.network.download_retries = value.max(1);
    }
    if let Some(value) = env_u64("VEX_RETRY_BASE_DELAY_SECS") {
        settings.network.retry_base_delay = Duration::from_secs(value.max(1));
    }
    if let Some(value) = env_usize("VEX_MAX_CONCURRENT_DOWNLOADS") {
        settings.network.max_concurrent_downloads = value.max(1);
    }
    if let Some(value) = env_usize("VEX_MAX_HTTP_REDIRECTS") {
        settings.network.max_http_redirects = value.max(1);
    }
    if let Some(value) = env_string("VEX_PROXY") {
        settings.network.proxy = Some(value);
    }
    if let Some(value) = env_bool("VEX_AUTO_SWITCH") {
        settings.behavior.auto_switch = value;
    }
    if let Some(value) = env_bool("VEX_AUTO_ACTIVATE_VENV") {
        settings.behavior.auto_activate_venv = value;
    }
    if let Some(value) = env_string("VEX_DEFAULT_SHELL") {
        settings.behavior.default_shell = Some(value);
    }
    if let Some(value) = env_bool("VEX_NON_INTERACTIVE") {
        settings.behavior.non_interactive = value;
    }

    for (key, value) in std::env::vars() {
        if let Some(tool_name) = key.strip_prefix("VEX_MIRROR_") {
            if let Some(mirror) = non_empty(value) {
                settings
                    .mirrors
                    .insert(tool_name.to_ascii_lowercase(), mirror);
            }
        }
    }
}

fn env_u64(key: &str) -> Option<u64> {
    std::env::var(key).ok()?.parse().ok()
}

fn env_u32(key: &str) -> Option<u32> {
    std::env::var(key).ok()?.parse().ok()
}

fn env_usize(key: &str) -> Option<usize> {
    std::env::var(key).ok()?.parse().ok()
}

fn env_bool(key: &str) -> Option<bool> {
    match std::env::var(key).ok()?.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn env_string(key: &str) -> Option<String> {
    non_empty(std::env::var(key).ok()?)
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn validated_cache_ttl(secs: u64) -> Duration {
    if secs == 0 {
        Duration::from_secs(0)
    } else {
        Duration::from_secs(secs.clamp(MIN_CACHE_TTL.as_secs(), MAX_CACHE_TTL.as_secs()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn save_env(keys: &[&str]) -> Vec<(String, Option<String>)> {
        keys.iter()
            .map(|key| ((*key).to_string(), std::env::var(key).ok()))
            .collect()
    }

    fn restore_env(saved: Vec<(String, Option<String>)>) {
        for (key, value) in saved {
            if let Some(value) = value {
                std::env::set_var(&key, value);
            } else {
                std::env::remove_var(&key);
            }
        }
    }

    #[test]
    fn test_timeout_values() {
        assert_eq!(CONNECT_TIMEOUT.as_secs(), 30);
        assert_eq!(READ_TIMEOUT.as_secs(), 300);
    }

    #[test]
    fn test_buffer_sizes() {
        assert_eq!(DOWNLOAD_BUFFER_SIZE, 65536);
        assert_eq!(CHECKSUM_BUFFER_SIZE, 65536);
    }

    #[test]
    fn test_retry_config() {
        assert_eq!(MAX_DOWNLOAD_RETRIES, 3);
        assert_eq!(RETRY_BASE_DELAY.as_secs(), 1);
    }

    #[test]
    fn test_cache_ttl_defaults() {
        assert_eq!(CACHE_TTL.as_secs(), 300);
        assert!(MIN_CACHE_TTL < CACHE_TTL);
        assert!(CACHE_TTL < MAX_CACHE_TTL);
    }

    #[test]
    fn test_directory_names() {
        assert_eq!(VEX_DIR_NAME, ".vex");
        assert_eq!(TOOLCHAINS_DIR, "toolchains");
        assert_eq!(CURRENT_DIR, "current");
        assert_eq!(BIN_DIR, "bin");
        assert_eq!(CACHE_DIR, "cache");
    }

    #[test]
    fn test_vex_home() {
        assert!(vex_home().is_some());
    }

    #[test]
    fn test_subdirectories() {
        if let Some(home) = vex_home() {
            assert_eq!(toolchains_dir(), Some(home.join(TOOLCHAINS_DIR)));
            assert_eq!(current_dir(), Some(home.join(CURRENT_DIR)));
            assert_eq!(bin_dir(), Some(home.join(BIN_DIR)));
            assert_eq!(cache_dir(), Some(home.join(CACHE_DIR)));
        }
    }

    #[test]
    fn test_http_config_defaults() {
        let settings = Settings::default();
        assert_eq!(settings.network.max_http_redirects, 10);
        assert!(settings.network.max_http_redirects > 0);
        assert!(settings.network.max_concurrent_downloads > 0);
    }

    #[test]
    fn test_disk_space_config() {
        assert_eq!(MIN_FREE_SPACE_BYTES, 1536 * 1024 * 1024);
    }

    #[test]
    fn test_vex_home_with_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original = std::env::var("VEX_HOME").ok();

        std::env::set_var("VEX_HOME", "/tmp/test_vex");
        assert_eq!(vex_home(), Some(PathBuf::from("/tmp/test_vex")));

        if let Some(value) = original {
            std::env::set_var("VEX_HOME", value);
        } else {
            std::env::remove_var("VEX_HOME");
        }
    }

    #[test]
    fn test_load_settings_from_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
cache_ttl_secs = 120

[network]
connect_timeout_secs = 5
download_retries = 8

[behavior]
auto_switch = false
default_shell = "fish"

[mirrors]
node = "https://mirror.example.com/node"
"#,
        )
        .unwrap();

        let settings = load_settings_from_file(&path).unwrap();
        assert_eq!(settings.cache_ttl.as_secs(), 120);
        assert_eq!(settings.network.connect_timeout.as_secs(), 5);
        assert_eq!(settings.network.download_retries, 8);
        assert!(!settings.behavior.auto_switch);
        assert_eq!(settings.behavior.default_shell.as_deref(), Some("fish"));
        assert_eq!(
            settings.mirrors.get("node").map(String::as_str),
            Some("https://mirror.example.com/node")
        );
    }

    #[test]
    fn test_invalid_config_returns_error() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");
        std::fs::write(&path, "{{ invalid toml").unwrap();

        let err = load_settings_from_file(&path).unwrap_err();
        assert!(err.to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_cache_ttl_is_clamped() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");
        std::fs::write(&path, "cache_ttl_secs = 1\n").unwrap();

        let settings = load_settings_from_file(&path).unwrap();
        assert_eq!(settings.cache_ttl, MIN_CACHE_TTL);
    }

    #[test]
    fn test_zero_cache_ttl_disables_cache() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");
        std::fs::write(&path, "cache_ttl_secs = 0\n").unwrap();

        let settings = load_settings_from_file(&path).unwrap();
        assert_eq!(settings.cache_ttl, Duration::from_secs(0));
    }

    #[test]
    fn test_rewrite_download_url_uses_tool_mirror() {
        let temp = TempDir::new().unwrap();
        let config_dir = temp.path().join(".vex");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.toml"),
            r#"
[mirrors]
node = "https://mirror.example.com/cache"
"#,
        )
        .unwrap();

        let settings = load_settings_from_file(&config_dir.join("config.toml")).unwrap();
        let rewritten = rewrite_download_url_with_settings(
            &settings,
            "node",
            "https://nodejs.org/dist/v24.14.0/node-v24.14.0-darwin-arm64.tar.gz",
        )
        .unwrap();
        assert_eq!(
            rewritten,
            "https://mirror.example.com/cache/dist/v24.14.0/node-v24.14.0-darwin-arm64.tar.gz"
        );
    }

    #[test]
    fn test_load_effective_settings_applies_project_overrides() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original_home = std::env::var("HOME").ok();
        let saved_env = save_env(&[
            "VEX_CONNECT_TIMEOUT_SECS",
            "VEX_READ_TIMEOUT_SECS",
            "VEX_PROXY",
            "VEX_AUTO_SWITCH",
            "VEX_AUTO_ACTIVATE_VENV",
            "VEX_DEFAULT_SHELL",
            "VEX_MIRROR_NODE",
        ]);
        let temp = TempDir::new().unwrap();
        let project = temp.path().join("workspace/project");
        std::fs::create_dir_all(project.join("nested")).unwrap();
        std::fs::create_dir_all(temp.path().join(".vex")).unwrap();

        std::fs::write(
            temp.path().join(".vex/config.toml"),
            r#"
[network]
read_timeout_secs = 60

[behavior]
auto_switch = true
default_shell = "zsh"

[mirrors]
node = "https://global.example.com/node"
"#,
        )
        .unwrap();
        std::fs::write(
            project.join(".vex.toml"),
            r#"
[network]
connect_timeout_secs = 9
proxy = "http://proxy.project.internal:8080"

[behavior]
auto_switch = false
auto_activate_venv = false
default_shell = "bash"

[mirrors]
node = "https://project.example.com/node"
"#,
        )
        .unwrap();

        std::env::set_var("HOME", temp.path());
        for key in [
            "VEX_CONNECT_TIMEOUT_SECS",
            "VEX_READ_TIMEOUT_SECS",
            "VEX_PROXY",
            "VEX_AUTO_SWITCH",
            "VEX_AUTO_ACTIVATE_VENV",
            "VEX_DEFAULT_SHELL",
            "VEX_MIRROR_NODE",
        ] {
            std::env::remove_var(key);
        }
        let settings = load_effective_settings(&project.join("nested")).unwrap();

        assert_eq!(settings.network.connect_timeout.as_secs(), 9);
        assert_eq!(settings.network.read_timeout.as_secs(), 60);
        assert_eq!(
            settings.network.proxy.as_deref(),
            Some("http://proxy.project.internal:8080")
        );
        assert!(!settings.behavior.auto_switch);
        assert!(!settings.behavior.auto_activate_venv);
        assert_eq!(settings.behavior.default_shell.as_deref(), Some("bash"));
        assert_eq!(
            settings.mirrors.get("node").map(String::as_str),
            Some("https://project.example.com/node")
        );

        if let Some(value) = original_home {
            std::env::set_var("HOME", value);
        } else {
            std::env::remove_var("HOME");
        }
        restore_env(saved_env);
    }

    #[test]
    fn test_load_effective_settings_keeps_env_overrides_highest_priority() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original_home = std::env::var("HOME").ok();
        let saved_env = save_env(&["VEX_PROXY", "VEX_DOWNLOAD_RETRIES"]);
        let temp = TempDir::new().unwrap();
        let project = temp.path().join("workspace/project");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(temp.path().join(".vex")).unwrap();

        std::fs::write(
            temp.path().join(".vex/config.toml"),
            r#"
[network]
download_retries = 3
"#,
        )
        .unwrap();
        std::fs::write(
            project.join(".vex.toml"),
            r#"
[network]
download_retries = 7
proxy = "http://proxy.project.internal:8080"
"#,
        )
        .unwrap();

        std::env::set_var("HOME", temp.path());
        std::env::set_var("VEX_PROXY", "http://proxy.env.internal:8888");
        std::env::set_var("VEX_DOWNLOAD_RETRIES", "11");
        let settings = load_effective_settings(&project).unwrap();

        assert_eq!(settings.network.download_retries, 11);
        assert_eq!(
            settings.network.proxy.as_deref(),
            Some("http://proxy.env.internal:8888")
        );

        if let Some(value) = original_home {
            std::env::set_var("HOME", value);
        } else {
            std::env::remove_var("HOME");
        }
        restore_env(saved_env);
    }
}
