use super::layers::{non_empty, validated_cache_ttl};
use super::Settings;
use std::time::Duration;

pub(super) fn apply_env_overrides(settings: &mut Settings) {
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
    if let Some(value) = env_bool("VEX_CAPTURE_USER_STATE") {
        settings.behavior.capture_user_state = value;
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
