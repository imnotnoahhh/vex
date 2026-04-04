use std::collections::HashMap;
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
    pub capture_user_state: bool,
    pub default_shell: Option<String>,
    pub non_interactive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrictMode {
    Warn,
    Enforce,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrictSettings {
    pub home_hygiene: StrictMode,
    pub path_conflicts: StrictMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub cache_ttl: Duration,
    pub network: NetworkSettings,
    pub behavior: BehaviorSettings,
    pub strict: StrictSettings,
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
                capture_user_state: true,
                default_shell: None,
                non_interactive: false,
            },
            strict: StrictSettings {
                home_hygiene: StrictMode::Warn,
                path_conflicts: StrictMode::Warn,
            },
            mirrors: HashMap::new(),
        }
    }
}
