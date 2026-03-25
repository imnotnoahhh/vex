mod client;
#[cfg(test)]
mod parallel;
mod retry;

use crate::config;
use crate::error::Result;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

#[cfg(test)]
pub(super) fn create_http_client() -> Result<reqwest::blocking::Client> {
    client::create_http_client()
}

#[cfg(test)]
pub(super) fn download_file(url: &str, dest: &Path) -> Result<()> {
    let client = create_http_client()?;
    client::download_file_with_client(&client, url, dest)
}

pub(super) fn download_with_retry_with_settings(
    url: &str,
    dest: &Path,
    retries: u32,
    settings: &config::Settings,
) -> Result<()> {
    retry::download_with_retry_with_settings(url, dest, retries, settings)
}

#[cfg(test)]
pub(super) fn download_parallel(downloads: &[(String, PathBuf)], retries: u32) -> Result<()> {
    parallel::download_parallel(downloads, retries)
}
