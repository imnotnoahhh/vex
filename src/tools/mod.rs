//! Tool adapter layer module
//!
//! Defines [`Tool`] trait and language tool implementations (Node.js, Go, Java, Rust).
//! Provides architecture detection, version alias resolution, and fuzzy version matching.

use crate::error::Result;
use owo_colors::OwoColorize;

pub mod go;
pub mod java;
pub mod node;
pub mod rust;

/// CPU architecture enum (macOS supports ARM64 and x86_64)
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Arch {
    /// Apple Silicon (aarch64)
    Arm64,
    /// Intel (x86_64)
    X86_64,
}

impl Arch {
    /// Auto-detect current CPU architecture
    pub fn detect() -> Self {
        #[cfg(target_arch = "aarch64")]
        return Arch::Arm64;

        #[cfg(target_arch = "x86_64")]
        return Arch::X86_64;
    }
}

/// Tool version information
#[derive(Debug, Clone)]
pub struct Version {
    /// Version number (e.g., "v20.11.0", "1.23.5")
    pub version: String,
    /// LTS codename (e.g., Node.js "Iron", Java "LTS")
    pub lts: Option<String>,
}

/// Tool trait, all language tools must implement this interface
///
/// Provides version querying, download URL construction, checksum retrieval, binary file path mapping, etc.
pub trait Tool: Send + Sync {
    /// Return tool name (e.g., "node", "go", "java", "rust")
    fn name(&self) -> &str;
    /// Query remote available version list (descending by release time)
    fn list_remote(&self) -> Result<Vec<Version>>;
    /// Construct download URL for specified version and architecture
    fn download_url(&self, version: &str, arch: Arch) -> Result<String>;
    /// Construct checksum file URL, returns `None` if checksum is in API
    fn checksum_url(&self, version: &str, arch: Arch) -> Option<String>;
    /// Return list of executable file names provided by the tool
    fn bin_names(&self) -> Vec<&str>;
    /// Return path of bin directory relative to installation directory
    fn bin_subpath(&self) -> &str;

    /// Return (binary name, subpath) pairs, override when binaries are in different subdirectories (e.g., Rust)
    fn bin_paths(&self) -> Vec<(&str, &str)> {
        let subpath = self.bin_subpath();
        self.bin_names()
            .into_iter()
            .map(|name| (name, subpath))
            .collect()
    }

    /// Get SHA256 checksum for specified version, defaults to returning `None`
    fn get_checksum(&self, _version: &str, _arch: Arch) -> Result<Option<String>> {
        Ok(None)
    }

    /// Resolve version alias (e.g., "latest", "lts", "stable"), defaults to returning `None`
    fn resolve_alias(&self, _alias: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Post-install hook for tool-specific setup (e.g., Rust sysroot linking), defaults to no-op
    fn post_install(&self, _install_dir: &std::path::Path, _arch: Arch) -> Result<()> {
        Ok(())
    }
}

/// Get tool implementation by name, supports node, go, java, rust
pub fn get_tool(name: &str) -> Result<Box<dyn Tool>> {
    match name {
        "node" => Ok(Box::new(node::NodeTool)),
        "go" => Ok(Box::new(go::GoTool)),
        "java" => Ok(Box::new(java::JavaTool)),
        "rust" => Ok(Box::new(rust::RustTool)),
        _ => Err(crate::error::VexError::ToolNotFound(name.to_string())),
    }
}

/// Fuzzy version resolution: supports aliases (latest/lts/stable), partial version numbers (20→20.x), and exact versions
pub fn resolve_fuzzy_version(tool: &dyn Tool, partial: &str) -> Result<String> {
    // First, try alias resolution
    if let Some(resolved) = tool.resolve_alias(partial)? {
        return Ok(resolved);
    }

    // Check if it already looks like a full version (has 2+ dots like 20.11.0, or is a single number for java)
    let normalized = partial.strip_prefix('v').unwrap_or(partial);
    let dot_count = normalized.chars().filter(|c| *c == '.').count();

    // For Java, versions are single numbers (8, 11, 17, 21) — always exact
    // For others, 2+ dots means full version (20.11.0, 1.23.5)
    if tool.name() == "java" || dot_count >= 2 {
        return Ok(normalized.to_string());
    }

    // Partial version — query remote and prefix-match
    println!(
        "{}...",
        format!("Resolving {}@{}", tool.name(), partial).cyan()
    );
    let versions = tool.list_remote()?;
    let prefix = format!("{}.", normalized);

    versions
        .iter()
        .find(|v| {
            let ver = normalize_version(&v.version);
            ver == normalized || ver.starts_with(&prefix)
        })
        .map(|v| normalize_version(&v.version))
        .ok_or_else(|| crate::error::VexError::VersionNotFound {
            tool: tool.name().to_string(),
            version: partial.to_string(),
        })
}

/// Remove "v" prefix from version number
fn normalize_version(version: &str) -> String {
    version.strip_prefix('v').unwrap_or(version).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_detect() {
        let arch = Arch::detect();
        match arch {
            Arch::Arm64 | Arch::X86_64 => {}
        }
    }

    #[test]
    fn test_version_struct() {
        let v = Version {
            version: "20.11.0".to_string(),
            lts: Some("Iron".to_string()),
        };
        assert_eq!(v.version, "20.11.0");
        assert_eq!(v.lts, Some("Iron".to_string()));

        let v2 = Version {
            version: "22.0.0".to_string(),
            lts: None,
        };
        assert_eq!(v2.lts, None);
    }

    #[test]
    fn test_get_tool_valid() {
        for name in &["node", "go", "java", "rust"] {
            let tool = get_tool(name);
            assert!(tool.is_ok(), "get_tool({}) should succeed", name);
            assert_eq!(tool.unwrap().name(), *name);
        }
    }

    #[test]
    fn test_get_tool_invalid() {
        let result = get_tool("python");
        assert!(result.is_err());

        let result = get_tool("ruby");
        assert!(result.is_err());

        let result = get_tool("");
        assert!(result.is_err());
    }

    /// A mock tool for testing resolve_fuzzy_version with aliases
    struct MockTool {
        versions: Vec<Version>,
    }

    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock"
        }
        fn list_remote(&self) -> Result<Vec<Version>> {
            Ok(self.versions.clone())
        }
        fn download_url(&self, _version: &str, _arch: Arch) -> Result<String> {
            Ok(String::new())
        }
        fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
            None
        }
        fn bin_names(&self) -> Vec<&str> {
            vec!["mock"]
        }
        fn bin_subpath(&self) -> &str {
            "bin"
        }
        fn resolve_alias(&self, alias: &str) -> Result<Option<String>> {
            match alias {
                "latest" => Ok(self.versions.first().map(|v| v.version.clone())),
                "lts" => Ok(self
                    .versions
                    .iter()
                    .find(|v| v.lts.is_some())
                    .map(|v| v.version.clone())),
                _ => Ok(None),
            }
        }
    }

    #[test]
    fn test_resolve_fuzzy_version_alias_latest() {
        let tool = MockTool {
            versions: vec![
                Version {
                    version: "22.5.0".to_string(),
                    lts: None,
                },
                Version {
                    version: "20.11.0".to_string(),
                    lts: Some("Iron".to_string()),
                },
            ],
        };
        let result = resolve_fuzzy_version(&tool, "latest").unwrap();
        assert_eq!(result, "22.5.0");
    }

    #[test]
    fn test_resolve_fuzzy_version_alias_lts() {
        let tool = MockTool {
            versions: vec![
                Version {
                    version: "22.5.0".to_string(),
                    lts: None,
                },
                Version {
                    version: "20.11.0".to_string(),
                    lts: Some("Iron".to_string()),
                },
            ],
        };
        let result = resolve_fuzzy_version(&tool, "lts").unwrap();
        assert_eq!(result, "20.11.0");
    }

    #[test]
    fn test_resolve_fuzzy_version_unknown_alias_falls_through() {
        let tool = MockTool {
            versions: vec![Version {
                version: "22.5.0".to_string(),
                lts: None,
            }],
        };
        // "22.5.0" is a full version, should pass through
        let result = resolve_fuzzy_version(&tool, "22.5.0").unwrap();
        assert_eq!(result, "22.5.0");
    }

    #[test]
    fn test_default_resolve_alias_returns_none() {
        // Test that the default trait implementation returns None
        struct MinimalTool;
        impl Tool for MinimalTool {
            fn name(&self) -> &str {
                "minimal"
            }
            fn list_remote(&self) -> Result<Vec<Version>> {
                Ok(vec![])
            }
            fn download_url(&self, _: &str, _: Arch) -> Result<String> {
                Ok(String::new())
            }
            fn checksum_url(&self, _: &str, _: Arch) -> Option<String> {
                None
            }
            fn bin_names(&self) -> Vec<&str> {
                vec![]
            }
            fn bin_subpath(&self) -> &str {
                ""
            }
        }

        let tool = MinimalTool;
        assert_eq!(tool.resolve_alias("latest").unwrap(), None);
        assert_eq!(tool.resolve_alias("lts").unwrap(), None);
        // Test default get_checksum
        assert_eq!(tool.get_checksum("1.0", Arch::Arm64).unwrap(), None);
        // Test default post_install
        assert!(tool
            .post_install(std::path::Path::new("/tmp"), Arch::Arm64)
            .is_ok());
        // Test default bin_paths
        assert!(tool.bin_paths().is_empty());
    }

    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version("v20.11.0"), "20.11.0");
        assert_eq!(normalize_version("20.11.0"), "20.11.0");
        assert_eq!(normalize_version("v1.23"), "1.23");
        assert_eq!(normalize_version("1.23"), "1.23");
    }

    #[test]
    fn test_resolve_fuzzy_version_full_version() {
        let tool = MockTool {
            versions: vec![Version {
                version: "22.5.0".to_string(),
                lts: None,
            }],
        };
        // Full version with 2+ dots should pass through directly
        let result = resolve_fuzzy_version(&tool, "20.11.0").unwrap();
        assert_eq!(result, "20.11.0");
    }

    #[test]
    fn test_resolve_fuzzy_version_v_prefix() {
        let tool = MockTool {
            versions: vec![Version {
                version: "22.5.0".to_string(),
                lts: None,
            }],
        };
        // v-prefix should be stripped
        let result = resolve_fuzzy_version(&tool, "v20.11.0").unwrap();
        assert_eq!(result, "20.11.0");
    }

    #[test]
    fn test_resolve_fuzzy_version_partial_match() {
        let tool = MockTool {
            versions: vec![
                Version {
                    version: "v22.5.0".to_string(),
                    lts: None,
                },
                Version {
                    version: "v20.11.0".to_string(),
                    lts: Some("Iron".to_string()),
                },
            ],
        };
        // Partial "22" should match "22.5.0"
        let result = resolve_fuzzy_version(&tool, "22").unwrap();
        assert_eq!(result, "22.5.0");
    }

    #[test]
    fn test_resolve_fuzzy_version_no_match() {
        let tool = MockTool {
            versions: vec![Version {
                version: "22.5.0".to_string(),
                lts: None,
            }],
        };
        let result = resolve_fuzzy_version(&tool, "99");
        assert!(result.is_err());
    }
}
