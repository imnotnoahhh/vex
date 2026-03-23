use crate::error::{Result, VexError};
use crate::http;
use serde::Deserialize;

const GITHUB_API_LATEST: &str = "https://api.github.com/repos/imnotnoahhh/vex/releases/latest";

#[derive(Deserialize)]
pub(in crate::updater) struct GithubRelease {
    pub(in crate::updater) tag_name: String,
    pub(in crate::updater) assets: Vec<super::GithubAsset>,
}

#[derive(Deserialize)]
pub(in crate::updater) struct GithubAsset {
    pub(in crate::updater) name: String,
    pub(in crate::updater) browser_download_url: String,
}

pub(in crate::updater) fn fetch_latest_release() -> Result<GithubRelease> {
    let client = http::client_for_global_settings(concat!("vex/", env!("CARGO_PKG_VERSION")))?;

    let release: GithubRelease = client
        .get(GITHUB_API_LATEST)
        .send()
        .map_err(VexError::Network)?
        .json()
        .map_err(VexError::Network)?;

    Ok(release)
}
