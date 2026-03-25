use super::model::{CACHE_TTL, MAX_CACHE_TTL, MAX_DOWNLOAD_RETRIES, MIN_CACHE_TTL};
use super::*;
use std::sync::Mutex;
use std::time::Duration;
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
