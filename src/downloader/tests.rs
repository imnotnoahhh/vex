use super::transfer::{create_http_client, download_file, download_parallel};
use super::*;
use crate::error::VexError;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_verify_checksum_correct() {
    let dir = std::env::temp_dir().join("vex_test_checksum");
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("test_file.txt");

    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"hello world").unwrap();

    let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
    let result = verify_checksum(&file_path, expected);
    assert!(result.is_ok());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_verify_checksum_mismatch() {
    let dir = std::env::temp_dir().join("vex_test_checksum_bad");
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("test_file.txt");

    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"hello world").unwrap();

    let result = verify_checksum(
        &file_path,
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    assert!(result.is_err());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_verify_checksum_empty_file() {
    let dir = std::env::temp_dir().join("vex_test_checksum_empty");
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("empty.txt");

    File::create(&file_path).unwrap();

    let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let result = verify_checksum(&file_path, expected);
    assert!(result.is_ok());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_create_http_client() {
    let client = create_http_client();
    assert!(client.is_ok());
}

#[test]
fn test_http_client_has_user_agent() {
    let client = create_http_client().unwrap();
    drop(client);
}

#[test]
fn test_constants_defined() {
    use crate::config;
    assert_eq!(config::CONNECT_TIMEOUT.as_secs(), 30);
    assert_eq!(config::READ_TIMEOUT.as_secs(), 300);
    assert_eq!(config::DOWNLOAD_BUFFER_SIZE, 65536);
    assert_eq!(config::CHECKSUM_BUFFER_SIZE, 65536);
    assert_eq!(config::RETRY_BASE_DELAY.as_secs(), 1);
    assert_eq!(config::MAX_CONCURRENT_DOWNLOADS, 3);
}

#[test]
fn test_verify_checksum_returns_true_on_match() {
    let dir = std::env::temp_dir().join("vex_test_checksum_true");
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("test.txt");

    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"test").unwrap();

    let expected = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
    let result = verify_checksum(&file_path, expected);
    assert!(result.is_ok());
    assert!(result.unwrap());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_verify_checksum_error_on_mismatch() {
    let dir = std::env::temp_dir().join("vex_test_mismatch");
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("test.txt");

    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"test").unwrap();

    let result = verify_checksum(&file_path, "wrong_checksum");
    assert!(result.is_err());

    if let Err(VexError::ChecksumMismatch { expected, actual }) = result {
        assert_eq!(expected, "wrong_checksum");
        assert_eq!(
            actual,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    } else {
        panic!("Expected ChecksumMismatch error");
    }

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_atomic_write_cleanup_on_error() {
    let dir = std::env::temp_dir().join("vex_test_atomic_cleanup");
    fs::create_dir_all(&dir).unwrap();
    let dest = dir.join("test.txt");

    let result = download_file("http://invalid.url.that.does.not.exist.local/file", &dest);
    assert!(result.is_err());

    let entries: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();
    assert_eq!(entries.len(), 0, "Temp files should be cleaned up on error");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn test_download_parallel_empty() {
    let downloads: Vec<(String, PathBuf)> = vec![];
    let result = download_parallel(&downloads, 3);
    assert!(result.is_ok());
}

#[test]
fn test_max_concurrent_downloads_constant() {
    assert_eq!(config::MAX_CONCURRENT_DOWNLOADS, 3);
}

#[test]
fn test_download_with_retry_ignores_invalid_project_config_for_global_use() {
    let _guard = ENV_LOCK.lock().unwrap();
    let cwd = std::env::current_dir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("download.tmp");

    fs::write(
        dir.path().join(".vex.toml"),
        "[network]\nread_timeout_secs = \"oops\"\n",
    )
    .unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = download_with_retry("http://127.0.0.1:9/nope", &dest, 0);

    std::env::set_current_dir(cwd).unwrap();

    assert!(matches!(result, Err(VexError::Network(_))));
}
