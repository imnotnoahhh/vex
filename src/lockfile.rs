//! Lockfile support for reproducible toolchain setups
//!
//! Provides lockfile generation, parsing, and validation for frozen installs.
//! Lockfile format: `.tool-versions.lock` with exact versions and integrity data.

use crate::error::{Result, VexError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_lockfile_new() {
        let lockfile = Lockfile::new();
        assert_eq!(lockfile.version, 1);
        assert!(lockfile.tools.is_empty());
    }

    #[test]
    fn test_lockfile_add_tool() {
        let mut lockfile = Lockfile::new();
        lockfile.add_tool(
            "node".to_string(),
            LockEntry {
                version: "20.11.0".to_string(),
                sha256: Some("abc123".to_string()),
                url: None,
            },
        );
        assert_eq!(lockfile.tools.len(), 1);
        assert_eq!(lockfile.get_tool("node").unwrap().version, "20.11.0");
    }

    #[test]
    fn test_lockfile_serialization() {
        let mut lockfile = Lockfile::new();
        lockfile.add_tool(
            "node".to_string(),
            LockEntry {
                version: "20.11.0".to_string(),
                sha256: Some("abc123".to_string()),
                url: Some("https://example.com/node".to_string()),
            },
        );

        let serialized = lockfile.to_string().unwrap();
        let deserialized = Lockfile::from_str(&serialized).unwrap();

        assert_eq!(deserialized.version, 1);
        assert_eq!(deserialized.tools.len(), 1);
        let entry = deserialized.get_tool("node").unwrap();
        assert_eq!(entry.version, "20.11.0");
        assert_eq!(entry.sha256.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_lockfile_save_and_load() {
        let temp = TempDir::new().unwrap();
        let mut lockfile = Lockfile::new();
        lockfile.add_tool(
            "go".to_string(),
            LockEntry {
                version: "1.23.5".to_string(),
                sha256: None,
                url: None,
            },
        );

        let path = lockfile.save_to_dir(temp.path()).unwrap();
        assert!(path.exists());

        let loaded = Lockfile::load_from_dir(temp.path()).unwrap().unwrap();
        assert_eq!(loaded.tools.len(), 1);
        assert_eq!(loaded.get_tool("go").unwrap().version, "1.23.5");
    }

    #[test]
    fn test_lockfile_find_in_ancestors() {
        let temp = TempDir::new().unwrap();
        let parent = temp.path();
        let child = parent.join("subdir");
        fs::create_dir_all(&child).unwrap();

        let mut lockfile = Lockfile::new();
        lockfile.add_tool(
            "rust".to_string(),
            LockEntry {
                version: "1.93.1".to_string(),
                sha256: None,
                url: None,
            },
        );
        lockfile.save_to_dir(parent).unwrap();

        let found = Lockfile::find_in_ancestors(&child);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), parent.join(LOCKFILE_NAME));
    }

    #[test]
    fn test_lockfile_optional_fields() {
        let entry = LockEntry {
            version: "1.0.0".to_string(),
            sha256: None,
            url: None,
        };

        let mut lockfile = Lockfile::new();
        lockfile.add_tool("test".to_string(), entry);

        let serialized = lockfile.to_string().unwrap();
        // Optional fields should not appear in serialized output
        assert!(!serialized.contains("sha256"));
        assert!(!serialized.contains("url"));
    }
}
