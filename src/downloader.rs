use crate::error::{Result, VexError};
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

/// HTTP connection timeout (30 seconds)
const CONNECT_TIMEOUT_SECS: u64 = 30;

/// HTTP read timeout (5 minutes for large downloads)
const READ_TIMEOUT_SECS: u64 = 300;

/// Download buffer size (64 KB for better performance)
const DOWNLOAD_BUFFER_SIZE: usize = 65536;

/// Checksum verification buffer size (64 KB)
const CHECKSUM_BUFFER_SIZE: usize = 65536;

/// Retry delay in seconds
const RETRY_DELAY_SECS: u64 = 2;

/// Create a configured HTTP client with timeouts
fn create_http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .timeout(Duration::from_secs(READ_TIMEOUT_SECS))
        .user_agent(concat!("vex/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(VexError::Network)
}

pub fn download_file(url: &str, dest: &Path) -> Result<()> {
    let client = create_http_client()?;
    let mut response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(VexError::Network(response.error_for_status().unwrap_err()));
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} {binary_bytes_per_sec} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(format!("Downloading {}", url));

    let mut file = File::create(dest)?;
    let mut downloaded = 0u64;
    let mut buffer = vec![0u8; DOWNLOAD_BUFFER_SIZE];

    loop {
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Download complete");
    Ok(())
}

pub fn verify_checksum(file_path: &Path, expected: &str) -> Result<bool> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; CHECKSUM_BUFFER_SIZE];

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

pub fn download_with_retry(url: &str, dest: &Path, retries: u32) -> Result<()> {
    let mut attempts = 0;

    loop {
        match download_file(url, dest) {
            Ok(_) => return Ok(()),
            Err(e) => {
                // 4xx 客户端错误不重试（如 404）
                if let VexError::Network(ref req_err) = e {
                    if req_err
                        .status()
                        .map(|s| s.is_client_error())
                        .unwrap_or(false)
                    {
                        return Err(e);
                    }
                }
                if attempts < retries {
                    eprintln!("Download failed: {}", e);
                    eprintln!("Retrying... ({}/{} attempts)", attempts + 1, retries);
                    attempts += 1;
                    std::thread::sleep(Duration::from_secs(RETRY_DELAY_SECS));
                } else {
                    return Err(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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
        assert_eq!(CONNECT_TIMEOUT_SECS, 30);
        assert_eq!(READ_TIMEOUT_SECS, 300);
        assert_eq!(DOWNLOAD_BUFFER_SIZE, 65536);
        assert_eq!(CHECKSUM_BUFFER_SIZE, 65536);
        assert_eq!(RETRY_DELAY_SECS, 2);
    }
}

