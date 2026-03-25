use super::client::download_file_with_client;
use crate::config;
use crate::error::{Result, VexError};
use crate::http;
use std::path::Path;
use tracing::{error, info, warn};

pub(super) fn download_with_retry_with_settings(
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
            Err(error_value) => {
                if let VexError::Network(ref req_err) = error_value {
                    if req_err
                        .status()
                        .map(|status| status.is_client_error())
                        .unwrap_or(false)
                    {
                        error!("Client error, not retrying: {}", error_value);
                        return Err(error_value);
                    }
                }
                if attempts < retries {
                    warn!(
                        "Download failed (attempt {}/{}): {}",
                        attempts + 1,
                        retries,
                        error_value
                    );
                    eprintln!("Download failed: {}", error_value);
                    eprintln!("Retrying... ({}/{} attempts)", attempts + 1, retries);
                    attempts += 1;
                    std::thread::sleep(settings.network.retry_base_delay);
                } else {
                    error!("Download failed after {} attempts", retries);
                    return Err(error_value);
                }
            }
        }
    }
}
