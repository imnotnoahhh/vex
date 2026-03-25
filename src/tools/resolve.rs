mod cache;
mod suggest;

use super::{Tool, Version};
use crate::error::Result;
use crate::paths::vex_dir;
use owo_colors::OwoColorize;

pub(super) use crate::versioning::normalize_version;
use cache::fetch_versions_with_cache;
pub(super) use suggest::generate_version_suggestions;

/// Fuzzy version resolution with explicit cache control
///
/// # Arguments
/// - `tool` - Tool implementation
/// - `partial` - Version string (alias, partial, or full version)
/// - `use_cache` - Whether to use cached version lists (recommended: true)
pub(super) fn resolve_fuzzy_version_cached(
    tool: &dyn Tool,
    partial: &str,
    use_cache: bool,
) -> Result<String> {
    let normalized = partial.strip_prefix('v').unwrap_or(partial);
    let dot_count = normalized.chars().filter(|c| *c == '.').count();

    if tool.name() == "java" || dot_count >= 2 {
        println!(
            "{}...",
            format!("Validating {}@{}", tool.name(), partial).cyan()
        );
        let versions = fetch_versions_with_cache(tool, use_cache)?;
        let exists = versions
            .iter()
            .any(|v| normalize_version(&v.version) == normalized);

        if exists {
            return Ok(normalized.to_string());
        }

        let suggestions = generate_version_suggestions(normalized, &versions);
        return Err(crate::error::VexError::VersionNotFound {
            tool: tool.name().to_string(),
            version: partial.to_string(),
            suggestions,
        });
    }

    if let Ok(vex_dir) = vex_dir() {
        let alias_manager = crate::alias::AliasManager::new(&vex_dir);
        if let Ok(Some(version)) = alias_manager.resolve(tool.name(), partial) {
            return resolve_fuzzy_version_cached(tool, &version, use_cache);
        }
    }

    if let Some(resolved) = tool.resolve_alias(partial)? {
        return Ok(resolved);
    }

    println!(
        "{}...",
        format!("Resolving {}@{}", tool.name(), partial).cyan()
    );
    let versions = fetch_versions_with_cache(tool, use_cache)?;
    let prefix = format!("{}.", normalized);

    versions
        .iter()
        .find(|v| {
            let ver = normalize_version(&v.version);
            ver == normalized || ver.starts_with(&prefix)
        })
        .map(|v| normalize_version(&v.version))
        .ok_or_else(|| {
            let suggestions = generate_version_suggestions(normalized, &versions);
            crate::error::VexError::VersionNotFound {
                tool: tool.name().to_string(),
                version: partial.to_string(),
                suggestions,
            }
        })
}
