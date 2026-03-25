use super::AliasConfig;
use crate::error::{Result, VexError};
use std::fs;
use std::path::Path;

pub(super) fn load_config(path: &Path, label: &str) -> Result<AliasConfig> {
    if !path.exists() {
        return Ok(AliasConfig::default());
    }

    let content = fs::read_to_string(path)
        .map_err(|error| VexError::Config(format!("Failed to read {}: {}", label, error)))?;

    toml::from_str(&content)
        .map_err(|error| VexError::Config(format!("Failed to parse {}: {}", label, error)))
}

pub(super) fn save_config(path: &Path, config: &AliasConfig, label: &str) -> Result<()> {
    let content = toml::to_string_pretty(config)
        .map_err(|error| VexError::Config(format!("Failed to serialize aliases: {}", error)))?;

    fs::write(path, content)
        .map_err(|error| VexError::Config(format!("Failed to write {}: {}", label, error)))?;

    Ok(())
}
