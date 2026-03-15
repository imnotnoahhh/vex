//! Version alias management
use crate::error::{Result, VexError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

type AliasList = Vec<(String, Vec<(String, String)>)>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliasConfig {
    #[serde(flatten)]
    pub tools: HashMap<String, HashMap<String, String>>,
}

impl AliasConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| {
            VexError::Parse(format!(
                "Failed to parse alias file {}: {}",
                path.display(),
                e
            ))
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| VexError::Parse(format!("Failed to serialize aliases: {}", e)))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    pub fn set(&mut self, tool: &str, alias: &str, version: &str) {
        self.tools
            .entry(tool.to_string())
            .or_default()
            .insert(alias.to_string(), version.to_string());
    }

    pub fn get(&self, tool: &str, alias: &str) -> Option<&str> {
        self.tools
            .get(tool)
            .and_then(|aliases| aliases.get(alias))
            .map(|s| s.as_str())
    }

    pub fn delete(&mut self, tool: &str, alias: &str) -> bool {
        if let Some(aliases) = self.tools.get_mut(tool) {
            let removed = aliases.remove(alias).is_some();
            if aliases.is_empty() {
                self.tools.remove(tool);
            }
            removed
        } else {
            false
        }
    }

    pub fn list(&self, tool: &str) -> Vec<(String, String)> {
        self.tools
            .get(tool)
            .map(|aliases| {
                let mut items: Vec<_> = aliases
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                items.sort_by(|a, b| a.0.cmp(&b.0));
                items
            })
            .unwrap_or_default()
    }

    pub fn list_all(&self) -> AliasList {
        let mut result: Vec<_> = self
            .tools
            .iter()
            .map(|(tool, aliases)| {
                let mut items: Vec<_> = aliases
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                items.sort_by(|a, b| a.0.cmp(&b.0));
                (tool.clone(), items)
            })
            .collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
}

pub struct AliasManager {
    global_path: PathBuf,
    project_path: Option<PathBuf>,
}

impl AliasManager {
    pub fn new(vex_home: &Path) -> Self {
        let global_path = vex_home.join("aliases.toml");
        let project_path = Self::find_project_alias_file();
        Self {
            global_path,
            project_path,
        }
    }

    fn find_project_alias_file() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;
        loop {
            let candidate = current.join(".vex.toml");
            if candidate.exists() {
                return Some(candidate);
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    pub fn resolve(&self, tool: &str, alias: &str) -> Result<Option<String>> {
        if let Some(ref project_path) = self.project_path {
            let config = AliasConfig::load(project_path)?;
            if let Some(version) = config.get(tool, alias) {
                return Ok(Some(version.to_string()));
            }
        }
        let config = AliasConfig::load(&self.global_path)?;
        Ok(config.get(tool, alias).map(|s| s.to_string()))
    }

    pub fn set_global(&self, tool: &str, alias: &str, version: &str) -> Result<()> {
        let mut config = AliasConfig::load(&self.global_path)?;
        config.set(tool, alias, version);
        config.save(&self.global_path)
    }

    pub fn set_project(&self, tool: &str, alias: &str, version: &str) -> Result<()> {
        let project_path = std::env::current_dir()?.join(".vex.toml");
        let mut config = AliasConfig::load(&project_path)?;
        config.set(tool, alias, version);
        config.save(&project_path)
    }

    pub fn delete_global(&self, tool: &str, alias: &str) -> Result<bool> {
        let mut config = AliasConfig::load(&self.global_path)?;
        let removed = config.delete(tool, alias);
        if removed {
            config.save(&self.global_path)?;
        }
        Ok(removed)
    }

    pub fn delete_project(&self, tool: &str, alias: &str) -> Result<bool> {
        let project_path = std::env::current_dir()?.join(".vex.toml");
        if !project_path.exists() {
            return Ok(false);
        }
        let mut config = AliasConfig::load(&project_path)?;
        let removed = config.delete(tool, alias);
        if removed {
            config.save(&project_path)?;
        }
        Ok(removed)
    }

    pub fn list_global(&self, tool: Option<&str>) -> Result<AliasList> {
        let config = AliasConfig::load(&self.global_path)?;
        if let Some(tool) = tool {
            let aliases = config.list(tool);
            if aliases.is_empty() {
                Ok(vec![])
            } else {
                Ok(vec![(tool.to_string(), aliases)])
            }
        } else {
            Ok(config.list_all())
        }
    }

    pub fn list_project(&self, tool: Option<&str>) -> Result<AliasList> {
        let project_path = std::env::current_dir()?.join(".vex.toml");
        if !project_path.exists() {
            return Ok(vec![]);
        }
        let config = AliasConfig::load(&project_path)?;
        if let Some(tool) = tool {
            let aliases = config.list(tool);
            if aliases.is_empty() {
                Ok(vec![])
            } else {
                Ok(vec![(tool.to_string(), aliases)])
            }
        } else {
            Ok(config.list_all())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_alias_config_set_get() {
        let mut config = AliasConfig::default();
        config.set("node", "lts", "20.11.0");
        config.set("node", "stable", "21.0.0");
        config.set("go", "latest", "1.22.0");
        assert_eq!(config.get("node", "lts"), Some("20.11.0"));
        assert_eq!(config.get("node", "stable"), Some("21.0.0"));
        assert_eq!(config.get("go", "latest"), Some("1.22.0"));
        assert_eq!(config.get("node", "nonexistent"), None);
    }

    #[test]
    fn test_alias_config_delete() {
        let mut config = AliasConfig::default();
        config.set("node", "lts", "20.11.0");
        config.set("node", "stable", "21.0.0");
        assert!(config.delete("node", "lts"));
        assert_eq!(config.get("node", "lts"), None);
        assert!(!config.delete("node", "nonexistent"));
        assert!(config.delete("node", "stable"));
        assert!(!config.tools.contains_key("node"));
    }

    #[test]
    fn test_alias_config_list() {
        let mut config = AliasConfig::default();
        config.set("node", "lts", "20.11.0");
        config.set("node", "stable", "21.0.0");
        config.set("go", "latest", "1.22.0");
        let node_aliases = config.list("node");
        assert_eq!(node_aliases.len(), 2);
        let all_aliases = config.list_all();
        assert_eq!(all_aliases.len(), 2);
    }

    #[test]
    fn test_alias_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let alias_file = temp_dir.path().join("aliases.toml");
        let mut config = AliasConfig::default();
        config.set("node", "lts", "20.11.0");
        config.set("go", "latest", "1.22.0");
        config.save(&alias_file).unwrap();
        let loaded = AliasConfig::load(&alias_file).unwrap();
        assert_eq!(loaded.get("node", "lts"), Some("20.11.0"));
        assert_eq!(loaded.get("go", "latest"), Some("1.22.0"));
    }

    #[test]
    fn test_alias_manager_resolve_priority() {
        let temp_dir = TempDir::new().unwrap();
        let vex_home = temp_dir.path().join(".vex");
        fs::create_dir_all(&vex_home).unwrap();
        let global_path = vex_home.join("aliases.toml");
        let mut global_config = AliasConfig::default();
        global_config.set("node", "prod", "20.11.0");
        global_config.save(&global_path).unwrap();
        let project_dir = temp_dir.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let project_path = project_dir.join(".vex.toml");
        let mut project_config = AliasConfig::default();
        project_config.set("node", "prod", "21.0.0");
        project_config.save(&project_path).unwrap();
        let manager = AliasManager {
            global_path,
            project_path: Some(project_path),
        };
        let resolved = manager.resolve("node", "prod").unwrap();
        assert_eq!(resolved, Some("21.0.0".to_string()));
    }
}
