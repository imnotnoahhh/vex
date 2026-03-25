use super::{load_team_config, validate_remote_team_config_response, LoadedVersions};
use crate::error::{Result, VexError};
use crate::http;
use reqwest::header::CONTENT_TYPE;
use std::path::Path;

pub(super) fn load_https_team_config(url: &str, start_dir: &Path) -> Result<LoadedVersions> {
    let response = http::client_for_current_context(concat!("vex/", env!("CARGO_PKG_VERSION")))?
        .get(url)
        .send()
        .map_err(VexError::Network)?
        .error_for_status()
        .map_err(VexError::Network)?;

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let content = response.text().map_err(VexError::Network)?;

    validate_remote_team_config_response(url, content_type.as_deref(), &content)?;
    load_team_config(&content, url.to_string(), start_dir)
}
