use crate::config::model::StrictMode;
use crate::project;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Default)]
pub(in crate::config) struct FileConfig {
    pub(super) cache_ttl_secs: Option<u64>,
    #[serde(default)]
    pub(super) network: NetworkFileConfig,
    #[serde(default)]
    pub(super) behavior: BehaviorFileConfig,
    #[serde(default)]
    pub(super) strict: StrictFileConfig,
    #[serde(default)]
    pub(super) mirrors: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(super) struct NetworkFileConfig {
    pub(super) connect_timeout_secs: Option<u64>,
    pub(super) read_timeout_secs: Option<u64>,
    pub(super) download_retries: Option<u32>,
    pub(super) retry_base_delay_secs: Option<u64>,
    pub(super) max_concurrent_downloads: Option<usize>,
    pub(super) max_http_redirects: Option<usize>,
    pub(super) proxy: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(super) struct BehaviorFileConfig {
    pub(super) auto_switch: Option<bool>,
    pub(super) auto_activate_venv: Option<bool>,
    pub(super) capture_user_state: Option<bool>,
    pub(super) default_shell: Option<String>,
    pub(super) non_interactive: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(super) struct StrictFileConfig {
    pub(super) home_hygiene: Option<StrictModeDef>,
    pub(super) path_conflicts: Option<StrictModeDef>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum StrictModeDef {
    Warn,
    Enforce,
}

impl StrictModeDef {
    pub(super) fn into_model(self) -> StrictMode {
        match self {
            Self::Warn => StrictMode::Warn,
            Self::Enforce => StrictMode::Enforce,
        }
    }
}

pub(super) struct NetworkOverrides {
    pub(super) connect_timeout_secs: Option<u64>,
    pub(super) read_timeout_secs: Option<u64>,
    pub(super) download_retries: Option<u32>,
    pub(super) retry_base_delay_secs: Option<u64>,
    pub(super) max_concurrent_downloads: Option<usize>,
    pub(super) max_http_redirects: Option<usize>,
    pub(super) proxy: Option<String>,
}

impl From<NetworkFileConfig> for NetworkOverrides {
    fn from(config: NetworkFileConfig) -> Self {
        Self {
            connect_timeout_secs: config.connect_timeout_secs,
            read_timeout_secs: config.read_timeout_secs,
            download_retries: config.download_retries,
            retry_base_delay_secs: config.retry_base_delay_secs,
            max_concurrent_downloads: config.max_concurrent_downloads,
            max_http_redirects: config.max_http_redirects,
            proxy: config.proxy,
        }
    }
}

impl From<&project::ProjectNetworkConfig> for NetworkOverrides {
    fn from(config: &project::ProjectNetworkConfig) -> Self {
        Self {
            connect_timeout_secs: config.connect_timeout_secs,
            read_timeout_secs: config.read_timeout_secs,
            download_retries: config.download_retries,
            retry_base_delay_secs: config.retry_base_delay_secs,
            max_concurrent_downloads: config.max_concurrent_downloads,
            max_http_redirects: config.max_http_redirects,
            proxy: config.proxy.clone(),
        }
    }
}
