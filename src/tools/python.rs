//! Python tool implementation
//!
//! Uses python-build-standalone (astral-sh/python-build-standalone) GitHub releases
//! to provide prebuilt CPython binaries. Supports version aliases based on Python's
//! support lifecycle (bugfix, security, end-of-life).

mod aliases;
mod base;
mod install;
mod lifecycle;
mod releases;
#[cfg(test)]
mod tests;

use crate::error::{Result, VexError};
use crate::tools::{Arch, Tool, ToolEnvironment, Version};
use aliases::resolve_alias_from_versions;
use install::rewire_placeholder_binaries;
use lifecycle::{fallback_python_lifecycle_statuses, fetch_python_lifecycle_statuses};
use releases::{
    asset_filename, collect_available_versions, fetch_latest_release_tag, fetch_sha256sums,
    find_matching_checksum, lifecycle_status_for,
};
use std::collections::BTreeMap;
use tracing::warn;

pub use base::{
    base_bin_dir, base_env_dir, base_pip_bin, ensure_base_environment, is_base_env_healthy,
};

pub const PYTHON_BUILD_STANDALONE_INTERNAL_ALIAS: &str = "\u{1d70b}thon";

/// Python tool (python-build-standalone prebuilt CPython)
pub struct PythonTool;

impl Tool for PythonTool {
    fn name(&self) -> &str {
        "python"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        let tag = fetch_latest_release_tag()?;
        let content = fetch_sha256sums(&tag)?;
        let lifecycle_statuses = match fetch_python_lifecycle_statuses() {
            Ok(statuses) => statuses,
            Err(err) => {
                warn!(
                    "Falling back to built-in Python lifecycle statuses after official fetch failed: {}",
                    err
                );
                fallback_python_lifecycle_statuses()
            }
        };

        let versions = collect_available_versions(&content);

        let result = versions
            .into_iter()
            .map(|version| {
                let lifecycle = lifecycle_status_for(&version, &lifecycle_statuses);
                Version {
                    version,
                    lts: Some(lifecycle),
                }
            })
            .collect();

        Ok(result)
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        let tag = fetch_latest_release_tag()?;
        let filename = asset_filename(version, &tag, arch);
        let content = fetch_sha256sums(&tag)?;

        if find_matching_checksum(&content, &filename).is_some() {
            return Ok(format!(
                "https://github.com/astral-sh/python-build-standalone/releases/download/{}/{}",
                tag, filename
            ));
        }

        Err(VexError::VersionNotFound {
            tool: "python".to_string(),
            version: version.to_string(),
            suggestions: String::new(),
        })
    }

    fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
        // SHA256SUMS is a single file for all assets in the release
        // We'll handle it in get_checksum
        None
    }

    fn get_checksum(&self, version: &str, arch: Arch) -> Result<Option<String>> {
        let tag = fetch_latest_release_tag()?;
        let content = fetch_sha256sums(&tag)?;
        let filename = asset_filename(version, &tag, arch);
        Ok(find_matching_checksum(&content, &filename))
    }

    fn resolve_alias(&self, alias: &str) -> Result<Option<String>> {
        let versions = self.list_remote()?;
        Ok(resolve_alias_from_versions(&versions, alias))
    }

    fn bin_names(&self) -> Vec<&str> {
        vec![
            "python3",
            "pip3",
            "python",
            "pip",
            "2to3",
            "idle3",
            "pydoc3",
            "python3-config",
        ]
    }

    fn bin_subpath(&self) -> &str {
        "bin"
    }

    /// After extraction, replace empty placeholder files with symlinks to the
    /// versioned binaries (e.g. python3 → python3.12).
    /// python-build-standalone's install_only tarball ships python3, python,
    /// 2to3, idle3, pydoc3, python3-config as zero-byte placeholders.
    fn post_install(&self, install_dir: &std::path::Path, _arch: Arch) -> Result<()> {
        rewire_placeholder_binaries(install_dir)
    }

    fn post_switch(
        &self,
        vex_dir: &std::path::Path,
        install_dir: &std::path::Path,
        version: &str,
    ) -> Result<()> {
        ensure_base_environment(vex_dir, version, install_dir).map(|_| ())
    }

    fn link_dynamic_binaries(&self) -> bool {
        true
    }

    fn should_link_dynamic_binary(&self, name: &str) -> bool {
        name != PYTHON_BUILD_STANDALONE_INTERNAL_ALIAS
    }

    fn managed_environment(
        &self,
        vex_dir: &std::path::Path,
        install_dir: Option<&std::path::Path>,
    ) -> ToolEnvironment {
        let pip_cache = vex_dir.join("pip/cache");
        let managed_user_bin_dirs = install_dir
            .and_then(|path| path.file_name())
            .map(|version| {
                vec![base_bin_dir(vex_dir, &version.to_string_lossy())
                    .display()
                    .to_string()]
            })
            .unwrap_or_default();

        ToolEnvironment {
            managed_env: BTreeMap::from([(
                "PIP_CACHE_DIR".to_string(),
                pip_cache.display().to_string(),
            )]),
            managed_user_bin_dirs,
            owned_home_dirs: vec![
                pip_cache.display().to_string(),
                base::base_root(vex_dir).display().to_string(),
            ],
            project_owned_dirs: vec![".venv".to_string()],
        }
    }

    fn managed_env_keys(&self) -> Vec<&'static str> {
        vec!["PIP_CACHE_DIR"]
    }
}
