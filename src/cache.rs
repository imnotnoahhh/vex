//! Remote version list cache module
//!
//! Caches tool remote version lists to `~/.vex/cache/remote-<tool>.json`,
//! default TTL 300 seconds, configurable via `~/.vex/config.toml`.

use crate::tools::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize)]
struct CachedVersionEntry {
    version: String,
    lts: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CacheFile {
    versions: Vec<CachedVersionEntry>,
    cached_at: u64,
}

/// Remote version list cache manager
///
/// Serializes tool version lists as JSON and stores them in `~/.vex/cache/remote-<tool>.json`.
pub struct RemoteCache {
    cache_dir: PathBuf,
}

impl RemoteCache {
    /// Create cache manager
    ///
    /// # Arguments
    /// - `vex_dir` - vex root directory (`~/.vex`)
    pub fn new(vex_dir: &std::path::Path) -> Self {
        Self {
            cache_dir: vex_dir.join("cache"),
        }
    }

    fn cache_path(&self, tool_name: &str) -> PathBuf {
        self.cache_dir.join(format!("remote-{}.json", tool_name))
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get cached version list, returns `None` if TTL exceeded
    ///
    /// # Arguments
    /// - `tool_name` - Tool name
    /// - `ttl_secs` - Cache validity period (seconds)
    pub fn get_cached_versions(&self, tool_name: &str, ttl_secs: u64) -> Option<Vec<Version>> {
        let path = self.cache_path(tool_name);
        let data = fs::read_to_string(&path).ok()?;
        let cache: CacheFile = serde_json::from_str(&data).ok()?;

        let elapsed = Self::now_secs().saturating_sub(cache.cached_at);
        if elapsed > ttl_secs {
            return None;
        }

        let versions = cache
            .versions
            .into_iter()
            .map(|e| Version {
                version: e.version,
                lts: e.lts,
            })
            .collect();

        Some(versions)
    }

    /// Write version list to cache (silently ignores write failures)
    ///
    /// # Arguments
    /// - `tool_name` - Tool name
    /// - `versions` - Version list
    pub fn set_cached_versions(&self, tool_name: &str, versions: &[Version]) {
        let entries: Vec<CachedVersionEntry> = versions
            .iter()
            .map(|v| CachedVersionEntry {
                version: v.version.clone(),
                lts: v.lts.clone(),
            })
            .collect();

        let cache = CacheFile {
            versions: entries,
            cached_at: Self::now_secs(),
        };

        // Silently ignore write failures
        let _ = fs::create_dir_all(&self.cache_dir);
        if let Ok(json) = serde_json::to_string(&cache) {
            let _ = fs::write(self.cache_path(tool_name), json);
        }
    }
}

/// Read cache TTL from `~/.vex/config.toml` for tests.
#[cfg(test)]
pub fn read_cache_ttl(vex_dir: &std::path::Path) -> crate::error::Result<u64> {
    let config_path = vex_dir.join("config.toml");
    Ok(crate::config::load_settings_from_file(&config_path)?
        .cache_ttl
        .as_secs())
}
