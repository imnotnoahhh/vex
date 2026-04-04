use crate::error::{Result, VexError};
use crate::http;
use serde::{Deserialize, Serialize};
use toml::Value;

#[derive(Debug, Clone)]
pub struct ChannelManifest {
    raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageArtifact {
    pub package: String,
    pub target: String,
    pub url: String,
    pub checksum: String,
}

pub(crate) fn fetch_stable_version() -> Result<String> {
    fetch_channel_manifest("stable")?.version()
}

pub(crate) fn fetch_channel_manifest(version: &str) -> Result<ChannelManifest> {
    let content = http::get_text_in_current_context(
        &manifest_url(version),
        concat!("vex/", env!("CARGO_PKG_VERSION")),
    )?;
    parse_channel_manifest(&content)
}

pub(crate) fn manifest_url(version: &str) -> String {
    if version == "stable" {
        "https://static.rust-lang.org/dist/channel-rust-stable.toml".to_string()
    } else {
        format!(
            "https://static.rust-lang.org/dist/channel-rust-{}.toml",
            version
        )
    }
}

impl ChannelManifest {
    pub(crate) fn version(&self) -> Result<String> {
        let version = self
            .raw
            .get("pkg")
            .and_then(|pkg| pkg.get("rust"))
            .and_then(|rust| rust.get("version"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                VexError::Parse("Rust manifest is missing pkg.rust.version".to_string())
            })?;
        Ok(version
            .split_whitespace()
            .next()
            .unwrap_or(version)
            .to_string())
    }

    pub(crate) fn available_targets(&self) -> Vec<String> {
        self.raw
            .get("pkg")
            .and_then(|pkg| pkg.get("rust-std"))
            .and_then(|pkg| pkg.get("target"))
            .and_then(Value::as_table)
            .map(|targets| {
                targets
                    .iter()
                    .filter(|(_, value)| {
                        value.get("available").and_then(Value::as_bool) == Some(true)
                    })
                    .map(|(target, _)| target.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub(crate) fn available_components(&self, host_target: &str) -> Vec<String> {
        let Some(packages) = self.raw.get("pkg").and_then(Value::as_table) else {
            return Vec::new();
        };

        let mut components = packages
            .keys()
            .filter(|name| *name != "rust" && *name != "rust-std")
            .filter(|name| self.lookup_target_record(name, host_target).is_some())
            .map(|name| name.to_string())
            .collect::<Vec<_>>();
        components.sort();
        components
    }

    pub(crate) fn artifact_for_target(&self, target: &str) -> Result<PackageArtifact> {
        self.package_artifact("rust-std", target)
    }

    pub(crate) fn artifact_for_component(
        &self,
        component: &str,
        host_target: &str,
    ) -> Result<PackageArtifact> {
        self.package_artifact(component, host_target)
    }

    fn package_artifact(&self, package: &str, target: &str) -> Result<PackageArtifact> {
        let record = self.lookup_target_record(package, target).ok_or_else(|| {
            VexError::VersionNotFound {
                tool: "rust".to_string(),
                version: package.to_string(),
                suggestions: format!(" (target/component '{}' is not available)", target),
            }
        })?;

        let url = record
            .get("xz_url")
            .and_then(Value::as_str)
            .or_else(|| record.get("url").and_then(Value::as_str))
            .ok_or_else(|| {
                VexError::Parse(format!(
                    "Rust manifest missing download URL for {}",
                    package
                ))
            })?;
        let checksum = record
            .get("xz_hash")
            .and_then(Value::as_str)
            .or_else(|| record.get("hash").and_then(Value::as_str))
            .ok_or_else(|| {
                VexError::Parse(format!("Rust manifest missing checksum for {}", package))
            })?;

        Ok(PackageArtifact {
            package: package.to_string(),
            target: target.to_string(),
            url: url.to_string(),
            checksum: checksum.to_string(),
        })
    }

    fn lookup_target_record(&self, package: &str, target: &str) -> Option<&toml::Value> {
        let pkg = self.raw.get("pkg")?.get(package)?;
        let targets = pkg.get("target")?.as_table()?;
        let record = targets.get(target).or_else(|| targets.get("*"))?;
        if record.get("available").and_then(Value::as_bool) == Some(true) {
            Some(record)
        } else {
            None
        }
    }
}

fn parse_channel_manifest(content: &str) -> Result<ChannelManifest> {
    let raw: Value = toml::from_str(content)
        .map_err(|err| VexError::Parse(format!("Failed to parse Rust manifest: {}", err)))?;
    Ok(ChannelManifest { raw })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"
[pkg.rust]
version = "1.90.0 (abc 2026-01-01)"

[pkg.rust-std.target.aarch64-apple-ios]
available = true
xz_url = "https://example.com/rust-std-aarch64-apple-ios.tar.xz"
xz_hash = "std-ios-hash"

[pkg.rust-std.target.aarch64-apple-ios-sim]
available = true
xz_url = "https://example.com/rust-std-aarch64-apple-ios-sim.tar.xz"
xz_hash = "std-ios-sim-hash"

[pkg.rust-src.target."*"]
available = true
xz_url = "https://example.com/rust-src.tar.xz"
xz_hash = "rust-src-hash"

[pkg.clippy-preview.target.aarch64-apple-darwin]
available = true
xz_url = "https://example.com/clippy-preview.tar.xz"
xz_hash = "clippy-hash"
"#;

    #[test]
    fn parses_version_targets_and_components() {
        let manifest = parse_channel_manifest(FIXTURE).unwrap();
        assert_eq!(manifest.version().unwrap(), "1.90.0");
        assert!(manifest
            .available_targets()
            .contains(&"aarch64-apple-ios".to_string()));
        assert!(manifest
            .available_components("aarch64-apple-darwin")
            .contains(&"rust-src".to_string()));
        assert!(manifest
            .available_components("aarch64-apple-darwin")
            .contains(&"clippy-preview".to_string()));
    }

    #[test]
    fn picks_xz_artifacts_for_targets_and_components() {
        let manifest = parse_channel_manifest(FIXTURE).unwrap();
        let target = manifest.artifact_for_target("aarch64-apple-ios").unwrap();
        assert_eq!(target.checksum, "std-ios-hash");
        let component = manifest
            .artifact_for_component("rust-src", "aarch64-apple-darwin")
            .unwrap();
        assert_eq!(component.url, "https://example.com/rust-src.tar.xz");
    }
}
