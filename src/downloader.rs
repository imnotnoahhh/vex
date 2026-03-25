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

mod transfer;

use crate::checksum;
use crate::config;
use crate::error::Result;
use std::path::Path;

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
    checksum::verify_sha256(file_path, expected).map(|_| true)
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
    transfer::download_with_retry_with_settings(url, dest, retries, &settings)
}

pub fn download_with_retry_in_current_context(url: &str, dest: &Path, retries: u32) -> Result<()> {
    let settings = config::load_effective_settings_for_current_dir()?;
    transfer::download_with_retry_with_settings(url, dest, retries, &settings)
}

#[cfg(test)]
mod tests;
