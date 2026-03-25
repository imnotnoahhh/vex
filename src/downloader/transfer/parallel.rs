use super::retry::download_with_retry_with_settings;
use crate::config;
use crate::error::{Result, VexError};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub(super) fn download_parallel(downloads: &[(String, PathBuf)], retries: u32) -> Result<()> {
    let errors = Arc::new(Mutex::new(Vec::new()));
    let settings = config::load_effective_settings_for_current_dir()?;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(settings.network.max_concurrent_downloads)
        .build()
        .map_err(|err| VexError::Parse(format!("Failed to create thread pool: {}", err)))?;

    pool.install(|| {
        downloads.par_iter().for_each(|(url, dest)| {
            if let Err(error) = download_with_retry_with_settings(url, dest, retries, &settings) {
                errors.lock().unwrap().push(format!("{}", error));
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
