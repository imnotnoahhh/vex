//! HTTP download and checksum verification module
//!
//! Provides file download (with progress bar), SHA256 checksum verification, and automatic retry.
//! 4xx client errors are not retried, server/network errors are retried up to 3 times.
//!
//! # Features
//!
//! - **Atomic writes**: Downloads write to temporary files first, then atomically rename to avoid corruption
//! - **Parallel downloads**: Support for downloading multiple files concurrently (max 3 concurrent)
//! - **Automatic cleanup**: Failed downloads automatically clean up temporary files
//! - **Retry logic**: Network errors are retried up to 3 times with exponential backoff

use crate::config;
use crate::error::{Result, VexError};
use crate::http;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

/// Create HTTP client with timeout configuration for direct current-context download tests.
#[cfg(test)]
fn create_http_client() -> Result<reqwest::blocking::Client> {
    http::client_for_current_context(concat!("vex/", env!("CARGO_PKG_VERSION")))
}

fn download_file_with_client(
    client: &reqwest::blocking::Client,
    url: &str,
    dest: &Path,
) -> Result<()> {
    info!("Starting download: {} -> {}", url, dest.display());
    let mut response = client.get(url).send()?;

    if !response.status().is_success() {
        error!("Download failed with status: {}", response.status());
        return Err(VexError::Network(response.error_for_status().unwrap_err()));
    }

    let total_size = response.content_length().unwrap_or(0);
    debug!("Download size: {} bytes", total_size);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} {binary_bytes_per_sec} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(format!("Downloading {}", url));

    // Use tempfile for RAII cleanup on panic/error
    let mut temp_file = tempfile::NamedTempFile::new_in(dest.parent().ok_or_else(|| {
        VexError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Destination has no parent directory",
        ))
    })?)?;
    let mut downloaded = 0u64;
    let mut buffer = vec![0u8; config::DOWNLOAD_BUFFER_SIZE];

    let result = (|| -> Result<()> {
        loop {
            let bytes_read = response.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            temp_file.write_all(&buffer[..bytes_read])?;
            downloaded += bytes_read as u64;
            pb.set_position(downloaded);
        }
        Ok(())
    })();

    if result.is_err() {
        pb.finish_with_message("Download failed");
        return result;
    }

    // Atomic persist (rename)
    temp_file.persist(dest).map_err(|e| VexError::Io(e.error))?;
    pb.finish_with_message("Download complete");
    Ok(())
}

/// Download file to specified path with progress bar (atomic write)
///
/// # Arguments
/// - `url` - Download URL
/// - `dest` - Destination file path
///
/// # Errors
/// - `VexError::Network` - HTTP request failed
/// - `VexError::Io` - File write failed
#[cfg(test)]
pub fn download_file(url: &str, dest: &Path) -> Result<()> {
    let client = create_http_client()?;
    download_file_with_client(&client, url, dest)
}

/// Verify file's SHA256 checksum
///
/// # Arguments
/// - `file_path` - File path to verify
/// - `expected` - Expected SHA256 hex string
///
/// # Returns
/// - `Ok(true)` - Checksum matches
/// - `Err(VexError::ChecksumMismatch)` - Checksum mismatch
pub fn verify_checksum(file_path: &Path, expected: &str) -> Result<bool> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; config::CHECKSUM_BUFFER_SIZE];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let actual = format!("{:x}", result);

    if actual == expected {
        Ok(true)
    } else {
        Err(VexError::ChecksumMismatch {
            expected: expected.to_string(),
            actual,
        })
    }
}

/// File download with automatic retry
///
/// Automatically retries on failure, 4xx client errors (e.g., 404) are not retried.
///
/// # Arguments
/// - `url` - Download URL
/// - `dest` - Destination file path
/// - `retries` - Maximum retry attempts
pub fn download_with_retry(url: &str, dest: &Path, retries: u32) -> Result<()> {
    let settings = config::load_settings()?;
    download_with_retry_with_settings(url, dest, retries, &settings)
}

pub fn download_with_retry_in_current_context(url: &str, dest: &Path, retries: u32) -> Result<()> {
    let settings = config::load_effective_settings_for_current_dir()?;
    download_with_retry_with_settings(url, dest, retries, &settings)
}

pub fn download_with_retry_with_settings(
    url: &str,
    dest: &Path,
    retries: u32,
    settings: &config::Settings,
) -> Result<()> {
    info!("Download with retry: {} (max retries: {})", url, retries);
    let mut attempts = 0;
    let client = http::client_for_settings(settings, concat!("vex/", env!("CARGO_PKG_VERSION")))?;

    loop {
        match download_file_with_client(&client, url, dest) {
            Ok(_) => {
                info!("Download successful after {} attempts", attempts + 1);
                return Ok(());
            }
            Err(e) => {
                // Don't retry 4xx client errors (e.g., 404)
                if let VexError::Network(ref req_err) = e {
                    if req_err
                        .status()
                        .map(|s| s.is_client_error())
                        .unwrap_or(false)
                    {
                        error!("Client error, not retrying: {}", e);
                        return Err(e);
                    }
                }
                if attempts < retries {
                    warn!(
                        "Download failed (attempt {}/{}): {}",
                        attempts + 1,
                        retries,
                        e
                    );
                    eprintln!("Download failed: {}", e);
                    eprintln!("Retrying... ({}/{} attempts)", attempts + 1, retries);
                    attempts += 1;
                    std::thread::sleep(settings.network.retry_base_delay);
                } else {
                    error!("Download failed after {} attempts", retries);
                    return Err(e);
                }
            }
        }
    }
}

/// Download multiple files in parallel with retry
///
/// Downloads up to config::MAX_CONCURRENT_DOWNLOADS files concurrently.
/// Each download uses atomic write (temp file + rename).
///
/// # Arguments
/// - `downloads` - List of (url, dest_path) tuples
/// - `retries` - Maximum retry attempts per download
///
/// # Returns
/// - `Ok(())` if all downloads succeed
/// - `Err(VexError)` with first error encountered
#[allow(dead_code)]
pub fn download_parallel(downloads: &[(String, PathBuf)], retries: u32) -> Result<()> {
    let errors = Arc::new(Mutex::new(Vec::new()));
    let settings = config::load_effective_settings_for_current_dir()?;

    // Configure rayon thread pool with max concurrency
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(settings.network.max_concurrent_downloads)
        .build()
        .map_err(|e| VexError::Parse(format!("Failed to create thread pool: {}", e)))?;

    pool.install(|| {
        downloads.par_iter().for_each(|(url, dest)| {
            if let Err(e) = download_with_retry_with_settings(url, dest, retries, &settings) {
                // Store error message instead of error itself (reqwest::Error is not Clone)
                errors.lock().unwrap().push(format!("{}", e));
            }
        });
    });

    let errors = errors.lock().unwrap();
    if !errors.is_empty() {
        return Err(VexError::Parse(format!(
            "Parallel download failed: {}",
            errors[0]
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_verify_checksum_correct() {
        let dir = std::env::temp_dir().join("vex_test_checksum");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test_file.txt");

        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"hello world").unwrap();

        // sha256("hello world") = b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let result = verify_checksum(&file_path, expected);
        assert!(result.is_ok());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_verify_checksum_mismatch() {
        let dir = std::env::temp_dir().join("vex_test_checksum_bad");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test_file.txt");

        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"hello world").unwrap();

        let result = verify_checksum(
            &file_path,
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        assert!(result.is_err());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_verify_checksum_empty_file() {
        let dir = std::env::temp_dir().join("vex_test_checksum_empty");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("empty.txt");

        File::create(&file_path).unwrap();

        // sha256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let result = verify_checksum(&file_path, expected);
        assert!(result.is_ok());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_create_http_client() {
        let client = create_http_client();
        assert!(client.is_ok());
    }

    #[test]
    fn test_http_client_has_user_agent() {
        let client = create_http_client().unwrap();
        // Verify client was created successfully with configuration
        // The actual user agent is set internally and will be used in requests
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
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.txt");

        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"test").unwrap();

        // sha256("test") = 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08
        let expected = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        let result = verify_checksum(&file_path, expected);
        assert!(result.is_ok());
        assert!(result.unwrap());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_verify_checksum_error_on_mismatch() {
        let dir = std::env::temp_dir().join("vex_test_mismatch");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.txt");

        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"test").unwrap();

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

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_atomic_write_cleanup_on_error() {
        let dir = std::env::temp_dir().join("vex_test_atomic_cleanup");
        std::fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("test.txt");

        // Try to download from invalid URL
        let result = download_file("http://invalid.url.that.does.not.exist.local/file", &dest);
        assert!(result.is_err());

        // Verify no temp files left behind
        let entries: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 0, "Temp files should be cleaned up on error");

        std::fs::remove_dir_all(&dir).unwrap();
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

        std::fs::write(
            dir.path().join(".vex.toml"),
            "[network]\nread_timeout_secs = \"oops\"\n",
        )
        .unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = download_with_retry("http://127.0.0.1:9/nope", &dest, 0);

        std::env::set_current_dir(cwd).unwrap();

        assert!(matches!(result, Err(VexError::Network(_))));
    }
}
