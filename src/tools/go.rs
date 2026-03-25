//! Go tool implementation
//!
//! Uses go.dev JSON API to query versions, checksums directly included in API response.

mod api;
mod dist;
#[cfg(test)]
mod tests;

use crate::error::Result;
use crate::tools::{Arch, Tool, Version};
use api::{checksum_for_release, fetch_releases, release_versions, resolve_alias_from_versions};
use dist::download_url as dist_download_url;

/// Go tool (go.dev official distribution)
pub struct GoTool;

impl Tool for GoTool {
    fn name(&self) -> &str {
        "go"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        Ok(release_versions(fetch_releases()?))
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        Ok(dist_download_url(version, arch))
    }

    fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
        None
    }

    fn bin_names(&self) -> Vec<&str> {
        vec!["go", "gofmt"]
    }

    fn bin_subpath(&self) -> &str {
        "bin"
    }

    fn get_checksum(&self, version: &str, arch: Arch) -> Result<Option<String>> {
        Ok(checksum_for_release(fetch_releases()?, version, arch))
    }

    fn resolve_alias(&self, alias: &str) -> Result<Option<String>> {
        match alias {
            "latest" => {
                let versions = self.list_remote()?;
                Ok(resolve_alias_from_versions(&versions, alias))
            }
            _ if dist::is_supported_minor_alias(alias) => {
                let versions = self.list_remote()?;
                Ok(resolve_alias_from_versions(&versions, alias))
            }
            _ => Ok(None),
        }
    }
}
