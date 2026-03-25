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
    assert_eq!(read_cache_ttl(tmp.path()).unwrap(), 300);
}

#[test]
fn test_read_cache_ttl_custom() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("config.toml"), "cache_ttl_secs = 60\n").unwrap();
    assert_eq!(read_cache_ttl(tmp.path()).unwrap(), 60);
}

#[test]
fn test_read_cache_ttl_invalid_toml() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("config.toml"), "{{bad toml").unwrap();
    assert!(read_cache_ttl(tmp.path()).is_err());
}

#[test]
fn test_cache_multiple_tools() {
    let tmp = TempDir::new().unwrap();
    let cache = RemoteCache::new(tmp.path());

    let node_versions = vec![Version {
        version: "20.0.0".to_string(),
        lts: None,
    }];
    let go_versions = vec![Version {
        version: "1.21.0".to_string(),
        lts: None,
    }];

    cache.set_cached_versions("node", &node_versions);
    cache.set_cached_versions("go", &go_versions);

    let cached_node = cache.get_cached_versions("node", 300).unwrap();
    let cached_go = cache.get_cached_versions("go", 300).unwrap();

    assert_eq!(cached_node[0].version, "20.0.0");
    assert_eq!(cached_go[0].version, "1.21.0");
}

#[test]
fn test_cache_overwrite() {
    let tmp = TempDir::new().unwrap();
    let cache = RemoteCache::new(tmp.path());

    let v1 = vec![Version {
        version: "1.0.0".to_string(),
        lts: None,
    }];
    let v2 = vec![Version {
        version: "2.0.0".to_string(),
        lts: None,
    }];

    cache.set_cached_versions("node", &v1);
    cache.set_cached_versions("node", &v2);

    let cached = cache.get_cached_versions("node", 300).unwrap();
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].version, "2.0.0");
}

#[test]
fn test_cache_empty_versions() {
    let tmp = TempDir::new().unwrap();
    let cache = RemoteCache::new(tmp.path());

    let empty: Vec<Version> = vec![];
    cache.set_cached_versions("node", &empty);

    let cached = cache.get_cached_versions("node", 300).unwrap();
    assert_eq!(cached.len(), 0);
}

#[test]
fn test_read_cache_ttl_zero() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("config.toml"), "cache_ttl_secs = 0\n").unwrap();
    assert_eq!(read_cache_ttl(tmp.path()).unwrap(), 0);
}

#[test]
fn test_read_cache_ttl_negative() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("config.toml"), "cache_ttl_secs = -1\n").unwrap();
    assert!(read_cache_ttl(tmp.path()).is_err());
}

#[test]
fn test_cache_with_lts_versions() {
    let tmp = TempDir::new().unwrap();
    let cache = RemoteCache::new(tmp.path());

    let versions = vec![
        Version {
            version: "18.0.0".to_string(),
            lts: Some("Hydrogen".to_string()),
        },
        Version {
            version: "20.0.0".to_string(),
            lts: Some("Iron".to_string()),
        },
        Version {
            version: "21.0.0".to_string(),
            lts: None,
        },
    ];

    cache.set_cached_versions("node", &versions);
    let cached = cache.get_cached_versions("node", 300).unwrap();

    assert_eq!(cached.len(), 3);
    assert_eq!(cached[0].lts, Some("Hydrogen".to_string()));
    assert_eq!(cached[1].lts, Some("Iron".to_string()));
    assert_eq!(cached[2].lts, None);
}

#[test]
fn test_cache_directory_creation() {
    let tmp = TempDir::new().unwrap();
    let cache_dir = tmp.path().join("cache");

    assert!(!cache_dir.exists());

    let cache = RemoteCache::new(tmp.path());
    cache.set_cached_versions("node", &sample_versions());

    assert!(cache_dir.exists());
    assert!(cache_dir.is_dir());
}

#[test]
fn test_cache_file_format() {
    let tmp = TempDir::new().unwrap();
    let cache = RemoteCache::new(tmp.path());

    cache.set_cached_versions("node", &sample_versions());

    let cache_file = tmp.path().join("cache/remote-node.json");
    assert!(cache_file.exists());

    let content = fs::read_to_string(&cache_file).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_object());
    assert!(parsed.get("versions").is_some());
    assert!(parsed.get("cached_at").is_some());
}
