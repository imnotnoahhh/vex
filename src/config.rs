//! Unified configuration management for vex
//!
//! This module centralizes all configuration constants and provides
//! a single source of truth for application settings.

use std::path::PathBuf;
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub const MIN_CACHE_TTL: Duration = Duration::from_secs(60);

/// Maximum cache TTL (1 hour)
#[allow(dead_code)]
pub const MAX_CACHE_TTL: Duration = Duration::from_secs(3600);

/// vex home directory name
pub const VEX_DIR_NAME: &str = ".vex";

/// Toolchains subdirectory name
#[allow(dead_code)]
pub const TOOLCHAINS_DIR: &str = "toolchains";

/// Current version symlink directory name
#[allow(dead_code)]
pub const CURRENT_DIR: &str = "current";

/// Binary symlinks directory name
#[allow(dead_code)]
pub const BIN_DIR: &str = "bin";

/// Cache directory name
#[allow(dead_code)]
pub const CACHE_DIR: &str = "cache";

/// Locks directory name
#[allow(dead_code)]
pub const LOCKS_DIR: &str = "locks";

/// Get vex home directory path
///
/// Returns `~/.vex` or the path specified by `VEX_HOME` environment variable.
pub fn vex_home() -> Option<PathBuf> {
    std::env::var("VEX_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|p| p.join(VEX_DIR_NAME)))
}

/// Get toolchains directory path
#[allow(dead_code)]
pub fn toolchains_dir() -> Option<PathBuf> {
    vex_home().map(|p| p.join(TOOLCHAINS_DIR))
}

/// Get current version symlink directory path
#[allow(dead_code)]
pub fn current_dir() -> Option<PathBuf> {
    vex_home().map(|p| p.join(CURRENT_DIR))
}

/// Get binary symlinks directory path
#[allow(dead_code)]
pub fn bin_dir() -> Option<PathBuf> {
    vex_home().map(|p| p.join(BIN_DIR))
}

/// Get cache directory path
#[allow(dead_code)]
pub fn cache_dir() -> Option<PathBuf> {
    vex_home().map(|p| p.join(CACHE_DIR))
}

/// Get locks directory path
#[allow(dead_code)]
pub fn locks_dir() -> Option<PathBuf> {
    vex_home().map(|p| p.join(LOCKS_DIR))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_cache_ttl() {
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
        assert_eq!(LOCKS_DIR, "locks");
    }

    #[test]
    fn test_vex_home() {
        // Should return a path (either from env or home dir)
        assert!(vex_home().is_some());
    }

    #[test]
    fn test_subdirectories() {
        if let Some(home) = vex_home() {
            assert_eq!(toolchains_dir(), Some(home.join(TOOLCHAINS_DIR)));
            assert_eq!(current_dir(), Some(home.join(CURRENT_DIR)));
            assert_eq!(bin_dir(), Some(home.join(BIN_DIR)));
            assert_eq!(cache_dir(), Some(home.join(CACHE_DIR)));
            assert_eq!(locks_dir(), Some(home.join(LOCKS_DIR)));
        }
    }

    #[test]
    fn test_http_config() {
        let redirects = MAX_HTTP_REDIRECTS;
        assert_eq!(redirects, 10);
        assert!(redirects > 0);
        assert!(redirects < 100);
    }

    #[test]
    fn test_disk_space_config() {
        let space = MIN_FREE_SPACE_BYTES;
        assert_eq!(space, 1536 * 1024 * 1024);
        assert!(space > 0);
    }

    #[test]
    fn test_concurrent_downloads() {
        let downloads = MAX_CONCURRENT_DOWNLOADS;
        assert_eq!(downloads, 3);
        assert!(downloads > 0);
        assert!(downloads <= 10);
    }

    #[test]
    fn test_vex_home_with_env() {
        // Test with VEX_HOME environment variable
        let original = std::env::var("VEX_HOME").ok();

        std::env::set_var("VEX_HOME", "/tmp/test_vex");
        assert_eq!(vex_home(), Some(PathBuf::from("/tmp/test_vex")));

        // Restore original
        if let Some(val) = original {
            std::env::set_var("VEX_HOME", val);
        } else {
            std::env::remove_var("VEX_HOME");
        }
    }
}
