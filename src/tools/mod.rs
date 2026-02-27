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
/// Supports: "20" → latest 20.x, "20.11" → latest 20.11.x, "lts" → latest LTS.
pub fn resolve_fuzzy_version(tool: &dyn Tool, partial: &str) -> Result<String> {
    // "lts" keyword
    if partial.eq_ignore_ascii_case("lts") {
        let versions = tool.list_remote()?;
        return versions
            .iter()
            .find(|v| v.lts.is_some())
            .map(|v| normalize_version(&v.version))
            .ok_or_else(|| {
                crate::error::VexError::Parse(format!("No LTS version found for {}", tool.name()))
            });
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
}
