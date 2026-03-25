use crate::commands::doctor::types::{CheckStatus, DoctorCheck};
use crate::config;
use crate::resolver;

pub(super) fn collect_effective_settings_check() -> DoctorCheck {
    let cwd = resolver::current_dir();
    let settings = match config::load_effective_settings(&cwd) {
        Ok(settings) => settings,
        Err(err) => {
            return DoctorCheck {
                id: "effective_settings".to_string(),
                status: CheckStatus::Warn,
                summary: "effective configuration could not be loaded".to_string(),
                details: vec![err.to_string()],
            };
        }
    };

    let mut details = vec![
        format!(
            "connect_timeout = {}s",
            settings.network.connect_timeout.as_secs()
        ),
        format!(
            "read_timeout = {}s",
            settings.network.read_timeout.as_secs()
        ),
        format!("download_retries = {}", settings.network.download_retries),
        format!(
            "max_concurrent_downloads = {}",
            settings.network.max_concurrent_downloads
        ),
        format!(
            "max_http_redirects = {}",
            settings.network.max_http_redirects
        ),
        format!(
            "proxy = {}",
            settings
                .network
                .proxy
                .clone()
                .unwrap_or_else(|| "(not set)".to_string())
        ),
        format!("mirror_count = {}", settings.mirrors.len()),
        format!("auto_switch = {}", settings.behavior.auto_switch),
        format!(
            "auto_activate_venv = {}",
            settings.behavior.auto_activate_venv
        ),
        format!("non_interactive = {}", settings.behavior.non_interactive),
    ];

    let invalid = collect_invalid_settings(&settings);
    if invalid.is_empty() {
        DoctorCheck {
            id: "effective_settings".to_string(),
            status: CheckStatus::Ok,
            summary: "effective configuration is valid".to_string(),
            details,
        }
    } else {
        details.extend(invalid);
        DoctorCheck {
            id: "effective_settings".to_string(),
            status: CheckStatus::Warn,
            summary: "effective configuration contains invalid values".to_string(),
            details,
        }
    }
}

fn collect_invalid_settings(settings: &config::Settings) -> Vec<String> {
    let mut invalid = Vec::new();

    if let Some(proxy) = &settings.network.proxy {
        if let Err(err) = reqwest::Proxy::all(proxy) {
            invalid.push(format!("Invalid proxy URL: {} ({})", proxy, err));
        }
    }

    for (tool, mirror) in &settings.mirrors {
        if let Err(err) = reqwest::Url::parse(mirror) {
            invalid.push(format!("Invalid mirror for {}: {} ({})", tool, mirror, err));
        }
    }

    invalid
}
