use super::*;
use sha2::{Digest, Sha256};
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_archive_cache_miss() {
    let tmp = TempDir::new().unwrap();
    let cache = ArchiveCache::new(tmp.path());

    assert!(!cache.has_archive("node", "20.11.0", "node-v20.11.0.tar.gz"));
    assert!(cache
        .get_archive("node", "20.11.0", "node-v20.11.0.tar.gz")
        .is_none());
}

#[test]
fn test_archive_cache_store_and_retrieve() {
    let tmp = TempDir::new().unwrap();
    let cache = ArchiveCache::new(tmp.path());

    let test_file = tmp.path().join("test.tar.gz");
    let mut file = fs::File::create(&test_file).unwrap();
    file.write_all(b"test content").unwrap();
    drop(file);

    let cached_path = cache
        .store_archive("node", "20.11.0", "node-v20.11.0.tar.gz", &test_file)
        .unwrap();

    assert!(cache.has_archive("node", "20.11.0", "node-v20.11.0.tar.gz"));

    let retrieved = cache
        .get_archive("node", "20.11.0", "node-v20.11.0.tar.gz")
        .unwrap();
    assert_eq!(retrieved, cached_path);

    let content = fs::read_to_string(&retrieved).unwrap();
    assert_eq!(content, "test content");
}

#[test]
fn test_archive_cache_remove_version() {
    let tmp = TempDir::new().unwrap();
    let cache = ArchiveCache::new(tmp.path());

    let test_file = tmp.path().join("test.tar.gz");
    fs::write(&test_file, b"test").unwrap();
    cache
        .store_archive("node", "20.11.0", "node-v20.11.0.tar.gz", &test_file)
        .unwrap();

    assert!(cache.has_archive("node", "20.11.0", "node-v20.11.0.tar.gz"));

    cache.remove_version("node", "20.11.0").unwrap();

    assert!(!cache.has_archive("node", "20.11.0", "node-v20.11.0.tar.gz"));
}

#[test]
fn test_list_cached_versions() {
    let tmp = TempDir::new().unwrap();
    let cache = ArchiveCache::new(tmp.path());

    assert_eq!(cache.list_cached_versions("node").unwrap().len(), 0);

    let test_file = tmp.path().join("test.tar.gz");
    fs::write(&test_file, b"test").unwrap();

    cache
        .store_archive("node", "20.11.0", "node-v20.11.0.tar.gz", &test_file)
        .unwrap();
    cache
        .store_archive("node", "22.0.0", "node-v22.0.0.tar.gz", &test_file)
        .unwrap();

    let versions = cache.list_cached_versions("node").unwrap();
    assert_eq!(versions.len(), 2);
    assert!(versions.contains(&"20.11.0".to_string()));
    assert!(versions.contains(&"22.0.0".to_string()));
}

#[test]
fn test_verify_checksum_success() {
    let tmp = TempDir::new().unwrap();
    let cache = ArchiveCache::new(tmp.path());

    let test_file = tmp.path().join("test.tar.gz");
    fs::write(&test_file, b"test content").unwrap();

    let mut hasher = Sha256::new();
    hasher.update(b"test content");
    let expected = format!("{:x}", hasher.finalize());

    assert!(cache.verify_checksum(&test_file, &expected).is_ok());
}

#[test]
fn test_verify_checksum_failure() {
    let tmp = TempDir::new().unwrap();
    let cache = ArchiveCache::new(tmp.path());

    let test_file = tmp.path().join("test.tar.gz");
    fs::write(&test_file, b"test content").unwrap();

    let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";
    assert!(cache.verify_checksum(&test_file, wrong_checksum).is_err());
}
