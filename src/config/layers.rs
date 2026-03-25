mod model;

use super::{BehaviorSettings, NetworkSettings, Settings, MAX_CACHE_TTL, MIN_CACHE_TTL};
use crate::error::{Result, VexError};
use crate::project;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

pub(in crate::config) use model::FileConfig;
use model::NetworkOverrides;

pub(super) fn read_file_config(path: &Path) -> Result<Option<FileConfig>> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(VexError::Io(err)),
    };

    toml::from_str(&content)
        .map(Some)
        .map_err(|err| VexError::Config(format!("Failed to parse {}: {}", path.display(), err)))
}

pub(super) fn apply_file_config(settings: &mut Settings, file_config: FileConfig) {
    let FileConfig {
        cache_ttl_secs,
        network,
        behavior,
        mirrors,
    } = file_config;

    if let Some(cache_ttl_secs) = cache_ttl_secs {
        settings.cache_ttl = validated_cache_ttl(cache_ttl_secs);
    }

    apply_network_overrides(&mut settings.network, NetworkOverrides::from(network));

    apply_behavior_overrides(
        &mut settings.behavior,
        behavior.auto_switch,
        behavior.auto_activate_venv,
        behavior.default_shell,
        behavior.non_interactive,
    );

    apply_mirror_overrides(&mut settings.mirrors, mirrors);
}

pub(super) fn apply_project_config(
    settings: &mut Settings,
    project_config: &project::ProjectConfig,
) {
    apply_network_overrides(
        &mut settings.network,
        NetworkOverrides::from(&project_config.network),
    );

    apply_behavior_overrides(
        &mut settings.behavior,
        project_config.behavior.auto_switch,
        project_config.behavior.auto_activate_venv,
        project_config.behavior.default_shell.clone(),
        project_config.behavior.non_interactive,
    );

    apply_mirror_overrides(&mut settings.mirrors, project_config.mirrors.clone());
}

pub(super) fn validated_cache_ttl(secs: u64) -> Duration {
    if secs == 0 {
        Duration::from_secs(0)
    } else {
        Duration::from_secs(secs.clamp(MIN_CACHE_TTL.as_secs(), MAX_CACHE_TTL.as_secs()))
    }
}

pub(super) fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn apply_network_overrides(network: &mut NetworkSettings, overrides: NetworkOverrides) {
    if let Some(secs) = overrides.connect_timeout_secs {
        network.connect_timeout = Duration::from_secs(secs.max(1));
    }
    if let Some(secs) = overrides.read_timeout_secs {
        network.read_timeout = Duration::from_secs(secs.max(1));
    }
    if let Some(retries) = overrides.download_retries {
        network.download_retries = retries.max(1);
    }
    if let Some(secs) = overrides.retry_base_delay_secs {
        network.retry_base_delay = Duration::from_secs(secs.max(1));
    }
    if let Some(concurrency) = overrides.max_concurrent_downloads {
        network.max_concurrent_downloads = concurrency.max(1);
    }
    if let Some(redirects) = overrides.max_http_redirects {
        network.max_http_redirects = redirects.max(1);
    }
    if let Some(proxy) = overrides.proxy {
        network.proxy = non_empty(proxy);
    }
}

fn apply_behavior_overrides(
    behavior: &mut BehaviorSettings,
    auto_switch: Option<bool>,
    auto_activate_venv: Option<bool>,
    default_shell: Option<String>,
    non_interactive: Option<bool>,
) {
    if let Some(auto_switch) = auto_switch {
        behavior.auto_switch = auto_switch;
    }
    if let Some(auto_activate_venv) = auto_activate_venv {
        behavior.auto_activate_venv = auto_activate_venv;
    }
    if let Some(default_shell) = default_shell {
        behavior.default_shell = non_empty(default_shell);
    }
    if let Some(non_interactive) = non_interactive {
        behavior.non_interactive = non_interactive;
    }
}

fn apply_mirror_overrides(mirrors: &mut HashMap<String, String>, entries: HashMap<String, String>) {
    for (tool, mirror) in entries {
        if let Some(mirror) = non_empty(mirror) {
            mirrors.insert(tool.trim().to_ascii_lowercase(), mirror);
        }
    }
}
