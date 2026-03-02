//! Version file resolution module
//!
//! Traverses upward from project directory to find version files (`.tool-versions`, `.node-version`, etc.).
//! `.tool-versions` has higher priority than language-specific files.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Traverse upward from start directory to find version mappings for all tools
///
/// `.tool-versions` has higher priority than language-specific files (`.node-version`, etc.).
/// First found version takes precedence (child directory over parent directory).
///
/// # Arguments
/// - `start_dir` - Directory to start searching from
pub fn resolve_versions(start_dir: &Path) -> HashMap<String, String> {
    let mut versions = HashMap::new();
    let mut dir = start_dir.to_path_buf();

    loop {
        // 1. Check .tool-versions (highest priority)
        let tool_versions = dir.join(".tool-versions");
        if tool_versions.is_file() {
            if let Ok(content) = fs::read_to_string(&tool_versions) {
                for (tool, version) in parse_tool_versions(&content) {
                    versions.entry(tool).or_insert(version);
                }
            }
        }

        // 2. Check language-specific version files
        for (file, tool) in TOOL_VERSION_FILES {
            let path = dir.join(file);
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    let version = content.trim().to_string();
                    if !version.is_empty() {
                        versions.entry(tool.to_string()).or_insert(version);
                    }
                }
            }
        }

        // Traverse upward
        if !dir.pop() {
            break;
        }
    }

    versions
}

/// Query version for a single tool (traverse upward from start directory)
///
/// # Arguments
/// - `tool_name` - Tool name (e.g., "node", "go")
/// - `start_dir` - Directory to start searching from
///
/// # Returns
/// - `Some(String)` - Found version number
/// - `None` - No version file found
#[allow(dead_code)]
pub fn resolve_version(tool_name: &str, start_dir: &Path) -> Option<String> {
    let mut dir = start_dir.to_path_buf();

    loop {
        // Check .tool-versions
        let tool_versions = dir.join(".tool-versions");
        if tool_versions.is_file() {
            if let Ok(content) = fs::read_to_string(&tool_versions) {
                for (tool, version) in parse_tool_versions(&content) {
                    if tool == tool_name {
                        return Some(version);
                    }
                }
            }
        }

        // Check tool-specific version files
        for (file, tool) in TOOL_VERSION_FILES {
            if *tool == tool_name {
                let path = dir.join(file);
                if path.is_file() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let version = content.trim().to_string();
                        if !version.is_empty() {
                            return Some(version);
                        }
                    }
                }
            }
        }

        if !dir.pop() {
            break;
        }
    }

    None
}

/// Get current working directory, fallback to "." on failure
pub fn current_dir() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Language-specific version file mappings
const TOOL_VERSION_FILES: &[(&str, &str)] = &[
    (".node-version", "node"),
    (".nvmrc", "node"),
    (".go-version", "go"),
    (".java-version", "java"),
    (".rust-toolchain", "rust"),
];

/// Parse .tool-versions file content
fn parse_tool_versions(content: &str) -> Vec<(String, String)> {
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
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_versions_basic() {
        let content = "node 20.11.0\ngo 1.23.5\njava 21\nrust 1.93.1\n";
        let result = parse_tool_versions(content);
        assert_eq!(
            result,
            vec![
                ("node".into(), "20.11.0".into()),
                ("go".into(), "1.23.5".into()),
                ("java".into(), "21".into()),
                ("rust".into(), "1.93.1".into()),
            ]
        );
    }

    #[test]
    fn test_parse_tool_versions_with_comments() {
        let content = "# project versions\nnode 20.11.0\n\n# Go version\ngo 1.23.5\n";
        let result = parse_tool_versions(content);
        assert_eq!(
            result,
            vec![
                ("node".into(), "20.11.0".into()),
                ("go".into(), "1.23.5".into()),
            ]
        );
    }

    #[test]
    fn test_parse_tool_versions_empty() {
        let result = parse_tool_versions("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_tool_versions_extra_whitespace() {
        let content = "  node   20.11.0  \n  go   1.23.5  ";
        let result = parse_tool_versions(content);
        assert_eq!(
            result,
            vec![
                ("node".into(), "20.11.0".into()),
                ("go".into(), "1.23.5".into()),
            ]
        );
    }

    #[test]
    fn test_parse_tool_versions_only_tool_no_version() {
        let content = "node\ngo 1.23.5";
        let result = parse_tool_versions(content);
        // Lines without version should be skipped
        assert_eq!(result, vec![("go".into(), "1.23.5".into())]);
    }

    #[test]
    fn test_resolve_version_from_file() {
        let dir = std::env::temp_dir().join("vex_test_resolve");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // 写入 .node-version
        fs::write(dir.join(".node-version"), "20.11.0\n").unwrap();

        let result = resolve_version("node", &dir);
        assert_eq!(result, Some("20.11.0".into()));

        // Non-existent tool
        let result = resolve_version("go", &dir);
        assert_eq!(result, None);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_version_tool_versions_priority() {
        let dir = std::env::temp_dir().join("vex_test_resolve_priority");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // .tool-versions takes priority over .node-version
        fs::write(dir.join(".tool-versions"), "node 22.0.0\n").unwrap();
        fs::write(dir.join(".node-version"), "20.11.0\n").unwrap();

        let result = resolve_version("node", &dir);
        assert_eq!(result, Some("22.0.0".into()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_versions_all() {
        let dir = std::env::temp_dir().join("vex_test_resolve_all");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join(".tool-versions"), "node 20.11.0\ngo 1.23.5\n").unwrap();
        fs::write(dir.join(".java-version"), "21\n").unwrap();

        let versions = resolve_versions(&dir);
        assert_eq!(versions.get("node"), Some(&"20.11.0".into()));
        assert_eq!(versions.get("go"), Some(&"1.23.5".into()));
        assert_eq!(versions.get("java"), Some(&"21".into()));
        assert_eq!(versions.get("rust"), None);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_version_parent_dir() {
        let parent = std::env::temp_dir().join("vex_test_parent");
        let child = parent.join("subdir");
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&child).unwrap();

        // Version file in parent directory
        fs::write(parent.join(".node-version"), "20.11.0\n").unwrap();

        let result = resolve_version("node", &child);
        assert_eq!(result, Some("20.11.0".into()));

        let _ = fs::remove_dir_all(&parent);
    }
}
