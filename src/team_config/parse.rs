use crate::error::{Result, VexError};
use std::collections::BTreeSet;

pub(super) fn validate_remote_team_config_response(
    url: &str,
    content_type: Option<&str>,
    content: &str,
) -> Result<()> {
    if let Some(content_type) = content_type {
        let normalized = content_type
            .split(';')
            .next()
            .unwrap_or(content_type)
            .trim()
            .to_ascii_lowercase();

        if normalized.contains("html") || normalized.contains("xhtml") {
            return Err(VexError::Config(format!(
                "URL '{}' returned HTML content ('{}') instead of a TOML team config. Check the URL or authentication requirements.",
                url, content_type
            )));
        }

        let supported = normalized == "application/toml"
            || normalized == "application/x-toml"
            || normalized == "text/x-toml"
            || normalized == "application/octet-stream"
            || (normalized.starts_with("text/")
                && !normalized.contains("html")
                && !normalized.contains("xml"));
        if !supported {
            return Err(VexError::Config(format!(
                "URL '{}' returned unsupported content type '{}' for team config. Expected TOML or plain text content.",
                url, content_type
            )));
        }
    }

    let trimmed = content.trim_start();
    if trimmed.starts_with("<!DOCTYPE html")
        || trimmed.starts_with("<html")
        || trimmed.starts_with("<?xml")
    {
        return Err(VexError::Config(format!(
            "URL '{}' returned HTML/XML content instead of a TOML team config. Check the URL or authentication requirements.",
            url
        )));
    }

    Ok(())
}

pub(super) fn parse_team_config(content: &str) -> Result<Vec<(String, String)>> {
    let value: toml::Value = toml::from_str(content)
        .map_err(|err| VexError::Config(format!("Failed to parse team config: {}", err)))?;
    let table = value.as_table().ok_or_else(|| {
        VexError::Config("Team config must be a TOML table with a [tools] section.".to_string())
    })?;

    let allowed: BTreeSet<_> = ["version", "tools"].into_iter().collect();
    let unexpected: Vec<_> = table
        .keys()
        .filter(|key| !allowed.contains(key.as_str()))
        .cloned()
        .collect();
    if !unexpected.is_empty() {
        return Err(VexError::Config(format!(
            "Team config contains unsupported top-level fields: {}",
            unexpected.join(", ")
        )));
    }

    let version = table
        .get("version")
        .and_then(|value| value.as_integer())
        .unwrap_or(1);
    if version != 1 {
        return Err(VexError::Config(format!(
            "Unsupported team config version {}. Expected version = 1.",
            version
        )));
    }

    let tools = table
        .get("tools")
        .and_then(|value| value.as_table())
        .ok_or_else(|| VexError::Config("Team config must define a [tools] table.".to_string()))?;

    let mut versions = Vec::new();
    for (tool, value) in tools {
        let Some(version) = value.as_str() else {
            return Err(VexError::Config(format!(
                "Team config tool '{}' must be a string value.",
                tool
            )));
        };
        if version.trim().is_empty() {
            return Err(VexError::Config(format!(
                "Team config tool '{}' must not be empty.",
                tool
            )));
        }
        versions.push((tool.clone(), version.trim().to_string()));
    }

    if versions.is_empty() {
        return Err(VexError::Config(
            "Team config [tools] table must not be empty.".to_string(),
        ));
    }

    versions.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(versions)
}
