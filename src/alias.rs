//! User-defined version aliases
//!
//! Supports both global aliases (~/.vex/aliases.toml) and project aliases (.vex.toml).
//! Project aliases take precedence over global aliases.

mod filter;
mod store;

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[cfg(test)]
mod tests;
use filter::filter_aliases;
use store::{load_config, save_config};

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
        load_config(&self.global_path, "global aliases")
    }

    /// Load project aliases
    fn load_project(&self) -> Result<AliasConfig> {
        load_config(&Self::project_path(), "project aliases")
    }

    /// Save global aliases
    fn save_global(&self, config: &AliasConfig) -> Result<()> {
        save_config(&self.global_path, config, "global aliases")
    }

    /// Save project aliases
    fn save_project(&self, config: &AliasConfig) -> Result<()> {
        save_config(&Self::project_path(), config, "project aliases")
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
        Ok(filter_aliases(config.tools, tool))
    }

    /// List project aliases (optionally filtered by tool)
    pub fn list_project(
        &self,
        tool: Option<&str>,
    ) -> Result<HashMap<String, HashMap<String, String>>> {
        let config = self.load_project()?;
        Ok(filter_aliases(config.tools, tool))
    }

    /// Resolve an alias to a version (project aliases take precedence)
    pub fn resolve(&self, tool: &str, alias: &str) -> Result<Option<String>> {
        let project = self.load_project()?;
        if let Some(version) = project.tools.get(tool).and_then(|a| a.get(alias)) {
            return Ok(Some(version.clone()));
        }

        let global = self.load_global()?;
        Ok(global.tools.get(tool).and_then(|a| a.get(alias)).cloned())
    }
}
