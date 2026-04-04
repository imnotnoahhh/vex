//! Go tool implementation
//!
//! Uses go.dev JSON API to query versions, checksums directly included in API response.

mod api;
mod dist;
#[cfg(test)]
mod tests;

use crate::error::Result;
use crate::tools::{Arch, Tool, ToolEnvironment, Version};
use api::{checksum_for_release, fetch_releases, release_versions, resolve_alias_from_versions};
use dist::download_url as dist_download_url;
use std::collections::BTreeMap;

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

    fn managed_environment(
        &self,
        vex_dir: &std::path::Path,
        install_dir: Option<&std::path::Path>,
    ) -> ToolEnvironment {
        let go_root = vex_dir.join("go");
        let go_bin = go_root.join("bin");
        let go_mod = go_root.join("pkg/mod");
        let go_cache = go_root.join("cache");
        let mut managed_env = BTreeMap::from([
            ("GOPATH".to_string(), go_root.display().to_string()),
            ("GOBIN".to_string(), go_bin.display().to_string()),
            ("GOMODCACHE".to_string(), go_mod.display().to_string()),
            ("GOCACHE".to_string(), go_cache.display().to_string()),
        ]);
        if let Some(install_dir) = install_dir {
            managed_env.insert("GOROOT".to_string(), install_dir.display().to_string());
        }

        ToolEnvironment {
            managed_env,
            managed_user_bin_dirs: vec![go_bin.display().to_string()],
            owned_home_dirs: vec![
                go_root.display().to_string(),
                go_bin.display().to_string(),
                go_mod.display().to_string(),
                go_cache.display().to_string(),
            ],
            project_owned_dirs: Vec::new(),
        }
    }

    fn managed_env_keys(&self) -> Vec<&'static str> {
        vec!["GOROOT", "GOPATH", "GOBIN", "GOMODCACHE", "GOCACHE"]
    }
}
