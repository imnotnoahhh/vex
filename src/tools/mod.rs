use crate::error::Result;

pub mod go;
pub mod java;
pub mod node;
pub mod rust;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Arch {
    Arm64,
    X86_64,
}

impl Arch {
    pub fn detect() -> Self {
        #[cfg(target_arch = "aarch64")]
        return Arch::Arm64;

        #[cfg(target_arch = "x86_64")]
        return Arch::X86_64;
    }
}

#[derive(Debug, Clone)]
pub struct Version {
    pub version: String,
    pub lts: Option<String>,
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn list_remote(&self) -> Result<Vec<Version>>;
    fn download_url(&self, version: &str, arch: Arch) -> Result<String>;
    fn checksum_url(&self, version: &str, arch: Arch) -> Option<String>;
    fn bin_names(&self) -> Vec<&str>;
    fn bin_subpath(&self) -> &str;

    /// Returns (bin_name, subpath) pairs. Override when binaries live in different subdirectories.
    fn bin_paths(&self) -> Vec<(&str, &str)> {
        let subpath = self.bin_subpath();
        self.bin_names()
            .into_iter()
            .map(|name| (name, subpath))
            .collect()
    }

    /// Get the expected SHA256 checksum for a version. Returns None if not available.
    fn get_checksum(&self, _version: &str, _arch: Arch) -> Result<Option<String>> {
        Ok(None)
    }

    /// Resolve a version alias (e.g., "latest", "lts") to a concrete version.
    /// Returns Ok(None) if the alias is not recognized.
    fn resolve_alias(&self, _alias: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Post-install hook called after extraction. Used for tool-specific setup.
    /// Default implementation does nothing.
    fn post_install(&self, _install_dir: &std::path::Path, _arch: Arch) -> Result<()> {
        Ok(())
    }
}

pub fn get_tool(name: &str) -> Result<Box<dyn Tool>> {
    match name {
        "node" => Ok(Box::new(node::NodeTool)),
        "go" => Ok(Box::new(go::GoTool)),
        "java" => Ok(Box::new(java::JavaTool)),
        "rust" => Ok(Box::new(rust::RustTool)),
        _ => Err(crate::error::VexError::ToolNotFound(name.to_string())),
    }
}

/// Resolve a partial version string to a full version by querying remote.
/// Supports aliases (latest, lts, stable, lts-<codename>), partial versions (20 → 20.x), and exact versions.
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
    println!("Resolving {}@{}...", tool.name(), partial);
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

/// Strip "v" prefix from version string
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
    }
}
