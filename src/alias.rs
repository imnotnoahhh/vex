//! User-defined version aliases
//!
//! Supports both global aliases (~/.vex/aliases.toml) and project aliases (.vex.toml).
//! Project aliases take precedence over global aliases.

use crate::error::{Result, VexError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Alias configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliasConfig {
    #[serde(flatten)]
    pub tools: HashMap<String, HashMap<String, String>>,
}

/// Manages user-defined aliases
pub struct AliasManager {
    global_path: PathBuf,
}

impl AliasManager {
    /// Create a new alias manager
    pub fn new(vex_dir: &Path) -> Self {
        Self {
            global_path: vex_dir.join("aliases.toml"),
        }
    }

    /// Get project alias file path (.vex.toml in current directory)
    fn project_path() -> PathBuf {
        PathBuf::from(".vex.toml")
    }

    /// Load global aliases
    fn load_global(&self) -> Result<AliasConfig> {
        if !self.global_path.exists() {
            return Ok(AliasConfig::default());
        }

        let content = fs::read_to_string(&self.global_path)
            .map_err(|e| VexError::Config(format!("Failed to read global aliases: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| VexError::Config(format!("Failed to parse global aliases: {}", e)))
    }

    /// Load project aliases
    fn load_project(&self) -> Result<AliasConfig> {
        let path = Self::project_path();
        if !path.exists() {
            return Ok(AliasConfig::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| VexError::Config(format!("Failed to read project aliases: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| VexError::Config(format!("Failed to parse project aliases: {}", e)))
    }

    /// Save global aliases
    fn save_global(&self, config: &AliasConfig) -> Result<()> {
        let content = toml::to_string_pretty(config)
            .map_err(|e| VexError::Config(format!("Failed to serialize aliases: {}", e)))?;

        fs::write(&self.global_path, content)
            .map_err(|e| VexError::Config(format!("Failed to write global aliases: {}", e)))?;

        Ok(())
    }

    /// Save project aliases
    fn save_project(&self, config: &AliasConfig) -> Result<()> {
        let path = Self::project_path();
        let content = toml::to_string_pretty(config)
            .map_err(|e| VexError::Config(format!("Failed to serialize aliases: {}", e)))?;

        fs::write(&path, content)
            .map_err(|e| VexError::Config(format!("Failed to write project aliases: {}", e)))?;

        Ok(())
    }

    /// Set a global alias
    pub fn set_global(&self, tool: &str, alias: &str, version: &str) -> Result<()> {
        let mut config = self.load_global()?;
        config
            .tools
            .entry(tool.to_string())
            .or_default()
            .insert(alias.to_string(), version.to_string());
        self.save_global(&config)
    }

    /// Set a project alias
    pub fn set_project(&self, tool: &str, alias: &str, version: &str) -> Result<()> {
        let mut config = self.load_project()?;
        config
            .tools
            .entry(tool.to_string())
            .or_default()
            .insert(alias.to_string(), version.to_string());
        self.save_project(&config)
    }

    /// Delete a global alias
    pub fn delete_global(&self, tool: &str, alias: &str) -> Result<bool> {
        let mut config = self.load_global()?;
        let removed = config
            .tools
            .get_mut(tool)
            .and_then(|aliases| aliases.remove(alias))
            .is_some();

        if removed {
            self.save_global(&config)?;
        }

        Ok(removed)
    }

    /// Delete a project alias
    pub fn delete_project(&self, tool: &str, alias: &str) -> Result<bool> {
        let mut config = self.load_project()?;
        let removed = config
            .tools
            .get_mut(tool)
            .and_then(|aliases| aliases.remove(alias))
            .is_some();

        if removed {
            self.save_project(&config)?;
        }

        Ok(removed)
    }

    /// List global aliases (optionally filtered by tool)
    pub fn list_global(
        &self,
        tool: Option<&str>,
    ) -> Result<HashMap<String, HashMap<String, String>>> {
        let config = self.load_global()?;
        Ok(match tool {
            Some(t) => config
                .tools
                .get(t)
                .map(|aliases| {
                    let mut map = HashMap::new();
                    map.insert(t.to_string(), aliases.clone());
                    map
                })
                .unwrap_or_default(),
            None => config.tools,
        })
    }

    /// List project aliases (optionally filtered by tool)
    pub fn list_project(
        &self,
        tool: Option<&str>,
    ) -> Result<HashMap<String, HashMap<String, String>>> {
        let config = self.load_project()?;
        Ok(match tool {
            Some(t) => config
                .tools
                .get(t)
                .map(|aliases| {
                    let mut map = HashMap::new();
                    map.insert(t.to_string(), aliases.clone());
                    map
                })
                .unwrap_or_default(),
            None => config.tools,
        })
    }

    /// Resolve an alias to a version (project aliases take precedence)
    pub fn resolve(&self, tool: &str, alias: &str) -> Result<Option<String>> {
        // Try project aliases first
        let project = self.load_project()?;
        if let Some(version) = project.tools.get(tool).and_then(|a| a.get(alias)) {
            return Ok(Some(version.clone()));
        }

        // Fall back to global aliases
        let global = self.load_global()?;
        Ok(global.tools.get(tool).and_then(|a| a.get(alias)).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_alias_resolution() {
        let temp = TempDir::new().unwrap();
        let manager = AliasManager::new(temp.path());

        // Change to temp directory for project alias tests
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Set global alias
        manager.set_global("node", "prod", "20.11.0").unwrap();
        assert_eq!(
            manager.resolve("node", "prod").unwrap(),
            Some("20.11.0".to_string())
        );

        // Project alias overrides global
        manager.set_project("node", "prod", "21.0.0").unwrap();
        assert_eq!(
            manager.resolve("node", "prod").unwrap(),
            Some("21.0.0".to_string())
        );

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_alias_deletion() {
        let temp = TempDir::new().unwrap();
        let manager = AliasManager::new(temp.path());

        manager.set_global("node", "test", "20.0.0").unwrap();
        assert!(manager.delete_global("node", "test").unwrap());
        assert!(!manager.delete_global("node", "test").unwrap());
    }
}
