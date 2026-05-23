//! Version file resolution module
//!
//! Traverses upward from project directory to find version files (`.tool-versions`, `.node-version`, etc.).
//! `.tool-versions` has higher priority than language-specific files.

mod discovery;

#[cfg(test)]
pub use discovery::resolve_version;
pub use discovery::{
    current_dir, find_project_source, read_tool_versions_file, resolve_local_tool_versions_only,
    resolve_project_versions, resolve_versions,
};

/// Language-specific version file mappings
pub(super) const TOOL_VERSION_FILES: &[(&str, &str)] = &[
    (".node-version", "node"),
    (".nvmrc", "node"),
    (".go-version", "go"),
    (".java-version", "java"),
    (".rust-toolchain", "rust"),
    ("rust-toolchain", "rust"),
    (".rust-toolchain.toml", "rust"),
    ("rust-toolchain.toml", "rust"),
    (".python-version", "python"),
];

/// Extract the toolchain channel from a `rust-toolchain.toml` file body.
///
/// Supports the rustup `[toolchain] channel = "..."` form. Returns `None` when the
/// file is malformed or the channel key is missing.
pub fn parse_rust_toolchain_toml(content: &str) -> Option<String> {
    let value: toml::Value = toml::from_str(content).ok()?;
    let channel = value.get("toolchain")?.get("channel")?.as_str()?.trim();
    (!channel.is_empty()).then(|| channel.to_string())
}

/// Parse .tool-versions file content
pub fn parse_tool_versions(content: &str) -> Vec<(String, String)> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut parts = line.split_whitespace();
            let tool = parts.next()?;
            let version = parts.next()?;
            Some((tool.to_string(), version.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests;
