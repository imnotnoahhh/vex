//! Rust tool implementation
//!
//! Parses `channel-rust-stable.toml` to get stable version information.
//! Installs complete toolchain (rustc, cargo, clippy, rustfmt, rust-analyzer, etc., 11 binaries),
//! `post_install` handles linking rust-std to sysroot and dynamic library path fixes.
//! Version-specific checksum verification uses Rust's `.sha256` sidecar files.

pub(crate) mod dist;
pub(crate) mod install;
pub(crate) mod manifest;
#[cfg(test)]
mod tests;

use crate::error::Result;
use crate::http;
use crate::tools::{Arch, Tool, ToolEnvironment, Version};
use dist::{
    checksum_url as dist_checksum_url, download_url as dist_download_url, parse_sha256_sidecar,
};
use install::link_runtime_components;
use manifest::fetch_stable_version;
use std::collections::BTreeMap;

/// Rust tool (official stable toolchain)
pub struct RustTool;

impl Tool for RustTool {
    fn name(&self) -> &str {
        "rust"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        let version = fetch_stable_version()?;
        Ok(vec![Version { version, lts: None }])
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        Ok(dist_download_url(version, arch))
    }

    fn checksum_url(&self, version: &str, arch: Arch) -> Option<String> {
        Some(dist_checksum_url(version, arch))
    }

    fn bin_names(&self) -> Vec<&str> {
        vec![
            "rustc",
            "rustdoc",
            "rust-gdb",
            "rust-gdbgui",
            "rust-lldb",
            "cargo",
            "rustfmt",
            "cargo-fmt",
            "cargo-clippy",
            "clippy-driver",
            "rust-analyzer",
        ]
    }

    fn bin_subpath(&self) -> &str {
        "rustc/bin"
    }

    fn bin_paths(&self) -> Vec<(&str, &str)> {
        vec![
            ("rustc", "rustc/bin"),
            ("rustdoc", "rustc/bin"),
            ("rust-gdb", "rustc/bin"),
            ("rust-gdbgui", "rustc/bin"),
            ("rust-lldb", "rustc/bin"),
            ("cargo", "cargo/bin"),
            ("rustfmt", "rustfmt-preview/bin"),
            ("cargo-fmt", "rustfmt-preview/bin"),
            ("cargo-clippy", "clippy-preview/bin"),
            ("clippy-driver", "clippy-preview/bin"),
            ("rust-analyzer", "rust-analyzer-preview/bin"),
        ]
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
        Ok(parse_sha256_sidecar(&content))
    }

    fn resolve_alias(&self, alias: &str) -> Result<Option<String>> {
        match alias {
            "latest" | "stable" => {
                let versions = self.list_remote()?;
                Ok(versions.first().map(|version| version.version.clone()))
            }
            _ => Ok(None),
        }
    }

    fn post_install(&self, install_dir: &std::path::Path, arch: Arch) -> Result<()> {
        link_runtime_components(install_dir, arch)
    }

    fn managed_environment(
        &self,
        vex_dir: &std::path::Path,
        _install_dir: Option<&std::path::Path>,
    ) -> ToolEnvironment {
        let cargo_home = vex_dir.join("cargo");
        ToolEnvironment {
            managed_env: BTreeMap::from([(
                "CARGO_HOME".to_string(),
                cargo_home.display().to_string(),
            )]),
            managed_user_bin_dirs: vec![cargo_home.join("bin").display().to_string()],
            owned_home_dirs: vec![cargo_home.display().to_string()],
            project_owned_dirs: vec!["target".to_string()],
        }
    }

    fn managed_env_keys(&self) -> Vec<&'static str> {
        vec!["CARGO_HOME"]
    }
}
