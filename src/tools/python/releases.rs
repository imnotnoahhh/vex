mod catalog;
mod fetch;

use crate::error::Result;
use crate::tools::Arch;
use reqwest::blocking::Client;
use std::collections::BTreeMap;

pub(in crate::tools::python) fn create_github_client() -> Result<Client> {
    fetch::create_github_client()
}

pub(in crate::tools::python) fn fetch_text_with_retry(
    client: &Client,
    url: &str,
) -> Result<String> {
    fetch::fetch_text_with_retry(client, url)
}

pub(in crate::tools::python) fn fetch_latest_release_tag() -> Result<String> {
    fetch::fetch_latest_release_tag()
}

pub(in crate::tools::python) fn fetch_sha256sums(tag: &str) -> Result<String> {
    fetch::fetch_sha256sums(tag)
}

pub(in crate::tools::python) fn asset_filename(version: &str, tag: &str, arch: Arch) -> String {
    catalog::asset_filename(version, tag, arch)
}

pub(in crate::tools::python) fn find_matching_checksum(
    content: &str,
    filename: &str,
) -> Option<String> {
    catalog::find_matching_checksum(content, filename)
}

#[cfg(test)]
pub(in crate::tools::python) fn extract_python_version(asset_name: &str) -> Option<String> {
    catalog::extract_python_version(asset_name)
}

#[cfg(test)]
pub(in crate::tools::python) fn get_major_minor(version: &str) -> String {
    catalog::get_major_minor(version)
}

pub(in crate::tools::python) fn collect_available_versions(content: &str) -> Vec<String> {
    catalog::collect_available_versions(content)
}

pub(in crate::tools::python) fn lifecycle_status_for(
    version: &str,
    lifecycle_statuses: &BTreeMap<String, String>,
) -> String {
    catalog::lifecycle_status_for(version, lifecycle_statuses)
}
