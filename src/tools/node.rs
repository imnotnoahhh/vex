//! Node.js tool implementation
//!
//! Uses nodejs.org official API to query versions, supports LTS aliases (`lts`, `lts-iron`, etc.).
//! Checksums obtained via SHASUMS256.txt file.

mod api;
mod dist;
#[cfg(test)]
mod tests;

use crate::error::Result;
use crate::http;
use crate::tools::{Arch, Tool, ToolEnvironment, Version};
use api::{fetch_releases, resolve_alias_from_versions, version_from_release};
use dist::{checksum_url as dist_checksum_url, download_url as dist_download_url, find_checksum};
use std::collections::BTreeMap;

/// Node.js tool (nodejs.org official distribution)
pub struct NodeTool;

impl Tool for NodeTool {
    fn name(&self) -> &str {
        "node"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        Ok(fetch_releases()?
            .into_iter()
            .map(version_from_release)
            .collect())
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        Ok(dist_download_url(version, arch))
    }

    fn checksum_url(&self, version: &str, _arch: Arch) -> Option<String> {
        Some(dist_checksum_url(version))
    }

    fn bin_names(&self) -> Vec<&str> {
        vec!["node", "npm", "npx"]
    }

    fn bin_subpath(&self) -> &str {
        "bin"
    }

    fn get_checksum(&self, version: &str, arch: Arch) -> Result<Option<String>> {
        let checksum_url = match self.checksum_url(version, arch) {
            Some(url) => url,
            None => return Ok(None),
        };

        let content = http::get_text_in_current_context(
            &checksum_url,
            concat!("vex/", env!("CARGO_PKG_VERSION")),
        )?;

        Ok(find_checksum(&content, version, arch))
    }

    fn resolve_alias(&self, alias: &str) -> Result<Option<String>> {
        match alias {
            "latest" | "lts" => {
                let versions = self.list_remote()?;
                Ok(resolve_alias_from_versions(&versions, alias))
            }
            _ if alias.starts_with("lts-") => {
                let versions = self.list_remote()?;
                Ok(resolve_alias_from_versions(&versions, alias))
            }
            _ => Ok(None),
        }
    }

    fn managed_environment(
        &self,
        vex_dir: &std::path::Path,
        _install_dir: Option<&std::path::Path>,
    ) -> ToolEnvironment {
        let npm_cache = vex_dir.join("npm/cache");
        let npm_prefix = vex_dir.join("npm/prefix");
        let corepack_home = vex_dir.join("corepack");
        let pnpm_home = vex_dir.join("pnpm");
        let yarn_cache = vex_dir.join("yarn/cache");
        let managed_env = BTreeMap::from([
            (
                "NPM_CONFIG_CACHE".to_string(),
                npm_cache.display().to_string(),
            ),
            (
                "NPM_CONFIG_PREFIX".to_string(),
                npm_prefix.display().to_string(),
            ),
            (
                "COREPACK_HOME".to_string(),
                corepack_home.display().to_string(),
            ),
            ("PNPM_HOME".to_string(), pnpm_home.display().to_string()),
            (
                "YARN_CACHE_FOLDER".to_string(),
                yarn_cache.display().to_string(),
            ),
        ]);

        ToolEnvironment {
            managed_env,
            managed_user_bin_dirs: vec![
                npm_prefix.join("bin").display().to_string(),
                pnpm_home.display().to_string(),
            ],
            owned_home_dirs: vec![
                npm_cache.display().to_string(),
                npm_prefix.display().to_string(),
                corepack_home.display().to_string(),
                pnpm_home.display().to_string(),
                yarn_cache.display().to_string(),
            ],
            project_owned_dirs: vec!["node_modules".to_string(), "dist".to_string()],
        }
    }

    fn managed_env_keys(&self) -> Vec<&'static str> {
        vec![
            "NPM_CONFIG_CACHE",
            "NPM_CONFIG_PREFIX",
            "COREPACK_HOME",
            "PNPM_HOME",
            "YARN_CACHE_FOLDER",
        ]
    }
}
