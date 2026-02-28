use crate::tools::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_TTL_SECS: u64 = 300;

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

pub struct RemoteCache {
    cache_dir: PathBuf,
}

impl RemoteCache {
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

/// Read cache_ttl_secs from ~/.vex/config.toml, falling back to default.
pub fn read_cache_ttl(vex_dir: &std::path::Path) -> u64 {
    let config_path = vex_dir.join("config.toml");
    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return DEFAULT_TTL_SECS,
    };
    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => return DEFAULT_TTL_SECS,
    };
    table
        .get("cache_ttl_secs")
        .and_then(|v| v.as_integer())
        .map(|v| v as u64)
        .unwrap_or(DEFAULT_TTL_SECS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    fn sample_versions() -> Vec<Version> {
        vec![
            Version {
                version: "20.11.0".to_string(),
                lts: Some("Iron".to_string()),
            },
            Version {
                version: "22.0.0".to_string(),
                lts: None,
            },
        ]
    }

    #[test]
    fn test_cache_write_and_read() {
        let tmp = TempDir::new().unwrap();
        let cache = RemoteCache::new(tmp.path());
        let versions = sample_versions();

        cache.set_cached_versions("node", &versions);
        let cached = cache.get_cached_versions("node", 300).unwrap();

        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].version, "20.11.0");
        assert_eq!(cached[0].lts, Some("Iron".to_string()));
        assert_eq!(cached[1].version, "22.0.0");
        assert_eq!(cached[1].lts, None);
    }

    #[test]
    fn test_cache_expired() {
        let tmp = TempDir::new().unwrap();
        let cache = RemoteCache::new(tmp.path());
        let versions = sample_versions();

        cache.set_cached_versions("go", &versions);
        // Sleep briefly then use a TTL of 1 second
        thread::sleep(Duration::from_secs(2));
        let cached = cache.get_cached_versions("go", 1);
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_missing_file() {
        let tmp = TempDir::new().unwrap();
        let cache = RemoteCache::new(tmp.path());
        let cached = cache.get_cached_versions("rust", 300);
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_invalid_json_degrades() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("cache");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::write(cache_dir.join("remote-java.json"), "not valid json!!!").unwrap();

        let cache = RemoteCache::new(tmp.path());
        let cached = cache.get_cached_versions("java", 300);
        assert!(cached.is_none());
    }

    #[test]
    fn test_read_cache_ttl_default() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(read_cache_ttl(tmp.path()), 300);
    }

    #[test]
    fn test_read_cache_ttl_custom() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("config.toml"), "cache_ttl_secs = 60\n").unwrap();
        assert_eq!(read_cache_ttl(tmp.path()), 60);
    }

    #[test]
    fn test_read_cache_ttl_invalid_toml() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("config.toml"), "{{bad toml").unwrap();
        assert_eq!(read_cache_ttl(tmp.path()), 300);
    }
}
