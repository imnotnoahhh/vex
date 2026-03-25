//! Lockfile support for reproducible toolchain setups
//!
//! Provides lockfile generation, parsing, and validation for frozen installs.
//! Lockfile format: `.tool-versions.lock` with exact versions and integrity data.

use crate::error::{Result, VexError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(test)]
mod tests;

const LOCKFILE_NAME: &str = ".tool-versions.lock";

/// Lockfile entry for a single tool version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockEntry {
    /// Exact version string
    pub version: String,
    /// Optional SHA256 checksum for integrity verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Optional download URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Lockfile structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Format version for future compatibility
    pub version: u32,
    /// Tool name -> lock entry mapping
    pub tools: HashMap<String, LockEntry>,
}

impl Lockfile {
    /// Create a new empty lockfile
    pub fn new() -> Self {
        Self {
            version: 1,
            tools: HashMap::new(),
        }
    }

    /// Add or update a tool entry
    pub fn add_tool(&mut self, tool: String, entry: LockEntry) {
        self.tools.insert(tool, entry);
    }

    /// Get a tool entry
    pub fn get_tool(&self, tool: &str) -> Option<&LockEntry> {
        self.tools.get(tool)
    }

    /// Parse lockfile from TOML content
    pub fn from_str(content: &str) -> Result<Self> {
        toml::from_str(content)
            .map_err(|e| VexError::Config(format!("Failed to parse lockfile: {}", e)))
    }

    /// Serialize lockfile to TOML
    pub fn to_string(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| VexError::Config(format!("Failed to serialize lockfile: {}", e)))
    }

    /// Load lockfile from a directory (looks for .tool-versions.lock)
    #[cfg(test)]
    pub fn load_from_dir(dir: &Path) -> Result<Option<Self>> {
        let path = dir.join(LOCKFILE_NAME);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)?;
        Ok(Some(Self::from_str(&content)?))
    }

    /// Save lockfile to a directory
    pub fn save_to_dir(&self, dir: &Path) -> Result<PathBuf> {
        let path = dir.join(LOCKFILE_NAME);
        let content = self.to_string()?;
        fs::write(&path, content)?;
        Ok(path)
    }

    /// Find lockfile by traversing upward from start directory
    pub fn find_in_ancestors(start_dir: &Path) -> Option<PathBuf> {
        let mut dir = start_dir.to_path_buf();
        loop {
            let candidate = dir.join(LOCKFILE_NAME);
            if candidate.exists() {
                return Some(candidate);
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    /// Load lockfile by traversing upward from start directory
    pub fn load_from_ancestors(start_dir: &Path) -> Result<Option<Self>> {
        if let Some(path) = Self::find_in_ancestors(start_dir) {
            let content = fs::read_to_string(&path)?;
            Ok(Some(Self::from_str(&content)?))
        } else {
            Ok(None)
        }
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}
