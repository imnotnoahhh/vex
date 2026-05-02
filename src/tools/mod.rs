//! Tool adapter layer module
//!
//! Defines [`Tool`] trait and language tool implementations (Node.js, Go, Java, Rust).
//! Provides architecture detection, version alias resolution, and fuzzy version matching.

use crate::error::{Result, VexError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub mod go;
pub mod java;
pub mod node;
pub mod python;
mod resolve;
pub mod rust;
#[cfg(test)]
mod tests;

/// CPU architecture enum (macOS supports ARM64 and x86_64)
#[derive(Debug, Clone, Copy)]
pub enum Arch {
    /// Apple Silicon (aarch64)
    Arm64,
    /// Intel (x86_64)
    X86_64,
}

impl Arch {
    /// Auto-detect current CPU architecture
    pub fn detect() -> Result<Self> {
        match std::env::consts::ARCH {
            "aarch64" => Ok(Arch::Arm64),
            "x86_64" => Ok(Arch::X86_64),
            other => Err(VexError::UnsupportedArchitecture(other.to_string())),
        }
    }
}

/// Tool version information
#[derive(Debug, Clone)]
pub struct Version {
    /// Version number (e.g., "v20.11.0", "1.23.5")
    pub version: String,
    /// LTS codename (e.g., Node.js "Iron", Java "LTS")
    pub lts: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolEnvironment {
    pub managed_env: BTreeMap<String, String>,
    pub managed_user_bin_dirs: Vec<String>,
    pub owned_home_dirs: Vec<String>,
    pub project_owned_dirs: Vec<String>,
}

/// Tool trait, all language tools must implement this interface
///
/// Provides version querying, download URL construction, checksum retrieval, binary file path mapping, etc.
pub trait Tool: Send + Sync {
    /// Return tool name (e.g., "node", "go", "java", "rust")
    fn name(&self) -> &str;
    /// Query remote available version list (descending by release time)
    fn list_remote(&self) -> Result<Vec<Version>>;
    /// Construct download URL for specified version and architecture
    fn download_url(&self, version: &str, arch: Arch) -> Result<String>;
    /// Construct checksum file URL, returns `None` if checksum is in API
    fn checksum_url(&self, version: &str, arch: Arch) -> Option<String>;
    /// Return list of executable file names provided by the tool
    fn bin_names(&self) -> Vec<&str>;
    /// Return path of bin directory relative to installation directory
    fn bin_subpath(&self) -> &str;

    /// Return (binary name, subpath) pairs, override when binaries are in different subdirectories (e.g., Rust)
    fn bin_paths(&self) -> Vec<(&str, &str)> {
        let subpath = self.bin_subpath();
        self.bin_names()
            .into_iter()
            .map(|name| (name, subpath))
            .collect()
    }

    /// Get SHA256 checksum for specified version, defaults to returning `None`
    fn get_checksum(&self, _version: &str, _arch: Arch) -> Result<Option<String>> {
        Ok(None)
    }

    /// Resolve version alias (e.g., "latest", "lts", "stable"), defaults to returning `None`
    fn resolve_alias(&self, _alias: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Post-install hook for tool-specific setup (e.g., Rust sysroot linking), defaults to no-op
    fn post_install(&self, _install_dir: &std::path::Path, _arch: Arch) -> Result<()> {
        Ok(())
    }

    /// Post-switch hook for tool-specific active-version setup, defaults to no-op.
    /// A failure rolls the switch back to the previous active version.
    fn post_switch(&self, _vex_dir: &Path, _install_dir: &Path, _version: &str) -> Result<()> {
        Ok(())
    }

    /// Whether executable files not declared by [`Tool::bin_paths`] should be linked into `~/.vex/bin`.
    fn link_dynamic_binaries(&self) -> bool {
        true
    }

    /// Return managed user-state directories and environment variables for this tool.
    fn managed_environment(&self, _vex_dir: &Path, _install_dir: Option<&Path>) -> ToolEnvironment {
        ToolEnvironment::default()
    }

    /// Return the environment keys this tool may set when active.
    fn managed_env_keys(&self) -> Vec<&'static str> {
        Vec::new()
    }
}

/// Get tool implementation by name, supports node, go, java, rust
pub fn get_tool(name: &str) -> Result<Box<dyn Tool>> {
    match name {
        "node" => Ok(Box::new(node::NodeTool)),
        "go" => Ok(Box::new(go::GoTool)),
        "java" => Ok(Box::new(java::JavaTool)),
        "python" => Ok(Box::new(python::PythonTool)),
        "rust" => Ok(Box::new(rust::RustTool)),
        _ => Err(crate::error::VexError::ToolNotFound(name.to_string())),
    }
}

/// Fuzzy version resolution: supports aliases (latest/lts/stable), partial version numbers (20→20.x), and exact versions
///
/// This function uses cached version lists by default to avoid repeated API calls.
pub fn resolve_fuzzy_version(tool: &dyn Tool, partial: &str) -> Result<String> {
    resolve_fuzzy_version_cached(tool, partial, true)
}

/// Fuzzy version resolution with explicit cache control
///
/// # Arguments
/// - `tool` - Tool implementation
/// - `partial` - Version string (alias, partial, or full version)
/// - `use_cache` - Whether to use cached version lists (recommended: true)
pub fn resolve_fuzzy_version_cached(
    tool: &dyn Tool,
    partial: &str,
    use_cache: bool,
) -> Result<String> {
    resolve::resolve_fuzzy_version_cached(tool, partial, use_cache)
}
