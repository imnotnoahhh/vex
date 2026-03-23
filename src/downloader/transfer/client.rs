use crate::config;
use crate::error::{Result, VexError};
#[cfg(test)]
use crate::http;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{Read, Write};
use std::path::Path;
use tracing::{debug, error, info};

/// Create HTTP client with timeout configuration for direct current-context download tests.
#[cfg(test)]
pub(super) fn create_http_client() -> Result<reqwest::blocking::Client> {
    http::client_for_current_context(concat!("vex/", env!("CARGO_PKG_VERSION")))
}

pub(super) fn download_file_with_client(
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

    let progress = ProgressBar::new(total_size);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} {binary_bytes_per_sec} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    progress.set_message(format!("Downloading {}", url));

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
            progress.set_position(downloaded);
        }
        Ok(())
    })();

    if result.is_err() {
        progress.finish_with_message("Download failed");
        return result;
    }

    temp_file
        .persist(dest)
        .map_err(|err| VexError::Io(err.error))?;
    progress.finish_with_message("Download complete");
    Ok(())
}
