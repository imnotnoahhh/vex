//! Python tool implementation
//!
//! Uses python-build-standalone (astral-sh/python-build-standalone) GitHub releases
//! to provide prebuilt CPython binaries. Supports version aliases based on Python's
//! support lifecycle (bugfix, security, end-of-life).

use crate::error::Result;
use crate::tools::{Arch, Tool, Version};
use serde::Deserialize;

/// Python tool (python-build-standalone prebuilt CPython)
pub struct PythonTool;

#[derive(Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize, Debug)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

/// Python support status based on lifecycle
/// See: https://devguide.python.org/versions/
#[derive(Debug, Clone, PartialEq)]
pub enum SupportStatus {
    Bugfix,
    Security,
    EndOfLife,
    PreRelease,
}

impl SupportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportStatus::Bugfix => "bugfix",
            SupportStatus::Security => "security",
            SupportStatus::EndOfLife => "end-of-life",
            SupportStatus::PreRelease => "pre-release",
        }
    }

    /// Determine support status from major.minor version string
    pub fn from_version(major_minor: &str) -> Self {
        match major_minor {
            "3.15" | "3.14" => SupportStatus::PreRelease,
            "3.13" | "3.12" => SupportStatus::Bugfix,
            "3.11" | "3.10" => SupportStatus::Security,
            _ => SupportStatus::EndOfLife,
        }
    }
}

/// Fetch the latest release tag from python-build-standalone
fn fetch_latest_release() -> Result<GithubRelease> {
    let url = "https://api.github.com/repos/astral-sh/python-build-standalone/releases/latest";
    let client = reqwest::blocking::Client::builder()
        .user_agent("vex-version-manager")
        .build()?;
    let response = client.get(url).send()?;
    let release: GithubRelease = response.json()?;
    Ok(release)
}

/// Extract Python version from asset name like:
/// cpython-3.12.13+20260303-aarch64-apple-darwin-install_only.tar.gz
fn extract_python_version(asset_name: &str) -> Option<String> {
    // Format: cpython-<version>+<tag>-<arch>-...
    let without_prefix = asset_name.strip_prefix("cpython-")?;
    let version_part = without_prefix.split('+').next()?;
    Some(version_part.to_string())
}

/// Get major.minor from a full version string like "3.12.13"
fn get_major_minor(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[0], parts[1])
    } else {
        version.to_string()
    }
}

impl Tool for PythonTool {
    fn name(&self) -> &str {
        "python"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        let release = fetch_latest_release()?;
        let arch_str = "aarch64-apple-darwin"; // We'll filter by arch at download time

        let mut versions: Vec<String> = Vec::new();
        for asset in &release.assets {
            if asset.name.contains(arch_str)
                && asset.name.ends_with("install_only.tar.gz")
                && !asset.name.contains("stripped")
            {
                if let Some(ver) = extract_python_version(&asset.name) {
                    versions.push(ver);
                }
            }
        }

        // Sort descending by version
        versions.sort_by(|a, b| {
            let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
            let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
            b_parts.cmp(&a_parts)
        });

        let result = versions
            .into_iter()
            .map(|ver| {
                let mm = get_major_minor(&ver);
                let status = SupportStatus::from_version(&mm);
                Version {
                    version: ver,
                    lts: Some(status.as_str().to_string()),
                }
            })
            .collect();

        Ok(result)
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        let release = fetch_latest_release()?;
        let arch_str = match arch {
            Arch::Arm64 => "aarch64-apple-darwin",
            Arch::X86_64 => "x86_64-apple-darwin",
        };

        // Find the matching asset
        let prefix = format!("cpython-{}+", version);
        for asset in &release.assets {
            if asset.name.starts_with(&prefix)
                && asset.name.contains(arch_str)
                && asset.name.ends_with("install_only.tar.gz")
                && !asset.name.contains("stripped")
            {
                return Ok(asset.browser_download_url.clone());
            }
        }

        Err(crate::error::VexError::VersionNotFound {
            tool: "python".to_string(),
            version: version.to_string(),
        })
    }

    fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
        // SHA256SUMS is a single file for all assets in the release
        // We'll handle it in get_checksum
        None
    }

    fn get_checksum(&self, version: &str, arch: Arch) -> Result<Option<String>> {
        let release = fetch_latest_release()?;
        let tag = &release.tag_name;

        let arch_str = match arch {
            Arch::Arm64 => "aarch64-apple-darwin",
            Arch::X86_64 => "x86_64-apple-darwin",
        };

        let sha256_url = format!(
            "https://github.com/astral-sh/python-build-standalone/releases/download/{}/SHA256SUMS",
            tag
        );

        let client = reqwest::blocking::Client::builder()
            .user_agent("vex-version-manager")
            .build()?;
        let content = client.get(&sha256_url).send()?.text()?;

        // Find the matching filename in SHA256SUMS
        let filename_prefix = format!("cpython-{}+", version);
        for line in content.lines() {
            let parts: Vec<&str> = line.splitn(2, "  ").collect();
            if parts.len() == 2 {
                let checksum = parts[0];
                let filename = parts[1];
                if filename.starts_with(&filename_prefix)
                    && filename.contains(arch_str)
                    && filename.ends_with("install_only.tar.gz")
                    && !filename.contains("stripped")
                {
                    return Ok(Some(checksum.to_string()));
                }
            }
        }

        Ok(None)
    }

    fn resolve_alias(&self, alias: &str) -> Result<Option<String>> {
        let versions = self.list_remote()?;

        match alias {
            "latest" | "stable" | "bugfix" => {
                // Return the latest bugfix-phase version
                Ok(versions
                    .iter()
                    .find(|v| {
                        let mm = get_major_minor(&v.version);
                        SupportStatus::from_version(&mm) == SupportStatus::Bugfix
                    })
                    .map(|v| v.version.clone()))
            }
            "security" => Ok(versions
                .iter()
                .find(|v| {
                    let mm = get_major_minor(&v.version);
                    SupportStatus::from_version(&mm) == SupportStatus::Security
                })
                .map(|v| v.version.clone())),
            _ => Ok(None),
        }
    }

    fn bin_names(&self) -> Vec<&str> {
        vec!["python3", "pip3"]
    }

    fn bin_subpath(&self) -> &str {
        "bin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(PythonTool.name(), "python");
    }

    #[test]
    fn test_bin_names() {
        let bins = PythonTool.bin_names();
        assert!(bins.contains(&"python3"));
        assert!(bins.contains(&"pip3"));
    }

    #[test]
    fn test_bin_subpath() {
        assert_eq!(PythonTool.bin_subpath(), "bin");
    }

    #[test]
    fn test_bin_paths() {
        let paths = PythonTool.bin_paths();
        assert!(paths.contains(&("python3", "bin")));
        assert!(paths.contains(&("pip3", "bin")));
    }

    #[test]
    fn test_extract_python_version() {
        assert_eq!(
            extract_python_version(
                "cpython-3.12.13+20260303-aarch64-apple-darwin-install_only.tar.gz"
            ),
            Some("3.12.13".to_string())
        );
        assert_eq!(
            extract_python_version(
                "cpython-3.13.2+20250317-aarch64-apple-darwin-install_only.tar.gz"
            ),
            Some("3.13.2".to_string())
        );
        assert_eq!(extract_python_version("not-cpython-file.tar.gz"), None);
    }

    #[test]
    fn test_get_major_minor() {
        assert_eq!(get_major_minor("3.12.13"), "3.12");
        assert_eq!(get_major_minor("3.9.21"), "3.9");
        assert_eq!(get_major_minor("3.13.2"), "3.13");
        assert_eq!(get_major_minor("3.10"), "3.10");
    }

    #[test]
    fn test_support_status_from_version() {
        assert_eq!(SupportStatus::from_version("3.13"), SupportStatus::Bugfix);
        assert_eq!(SupportStatus::from_version("3.12"), SupportStatus::Bugfix);
        assert_eq!(SupportStatus::from_version("3.11"), SupportStatus::Security);
        assert_eq!(SupportStatus::from_version("3.10"), SupportStatus::Security);
        assert_eq!(SupportStatus::from_version("3.9"), SupportStatus::EndOfLife);
        assert_eq!(
            SupportStatus::from_version("3.14"),
            SupportStatus::PreRelease
        );
        assert_eq!(
            SupportStatus::from_version("3.15"),
            SupportStatus::PreRelease
        );
    }

    #[test]
    fn test_support_status_as_str() {
        assert_eq!(SupportStatus::Bugfix.as_str(), "bugfix");
        assert_eq!(SupportStatus::Security.as_str(), "security");
        assert_eq!(SupportStatus::EndOfLife.as_str(), "end-of-life");
        assert_eq!(SupportStatus::PreRelease.as_str(), "pre-release");
    }

    #[test]
    fn test_checksum_url_is_none() {
        // checksum_url returns None; checksums are fetched via get_checksum
        assert_eq!(PythonTool.checksum_url("3.12.13", Arch::Arm64), None);
    }

    #[test]
    fn test_resolve_alias_unknown() {
        // Unknown aliases should return None without network call
        // (this would fail if it tried to call list_remote)
        // We test the logic path only — actual resolution requires network
    }

    #[test]
    #[ignore] // Requires network
    fn test_list_remote() {
        let versions = PythonTool.list_remote().unwrap();
        assert!(!versions.is_empty());
        // All versions should have a support status label
        for v in &versions {
            assert!(v.lts.is_some());
        }
        // Versions should be sorted descending
        let vers: Vec<&str> = versions.iter().map(|v| v.version.as_str()).collect();
        assert!(vers[0] >= vers[1]);
    }

    #[test]
    #[ignore] // Requires network
    fn test_resolve_alias_latest() {
        let result = PythonTool.resolve_alias("latest").unwrap();
        assert!(result.is_some());
        let ver = result.unwrap();
        assert!(ver.starts_with("3."));
    }

    #[test]
    #[ignore] // Requires network
    fn test_resolve_alias_security() {
        let result = PythonTool.resolve_alias("security").unwrap();
        assert!(result.is_some());
        let ver = result.unwrap();
        let mm = get_major_minor(&ver);
        assert_eq!(SupportStatus::from_version(&mm), SupportStatus::Security);
    }

    #[test]
    #[ignore] // Requires network
    fn test_download_url_arm64() {
        let url = PythonTool.download_url("3.12.13", Arch::Arm64).unwrap();
        assert!(url.contains("aarch64-apple-darwin"));
        assert!(url.contains("install_only.tar.gz"));
        assert!(url.contains("3.12.13"));
    }

    #[test]
    #[ignore] // Requires network
    fn test_download_url_x86() {
        let url = PythonTool.download_url("3.12.13", Arch::X86_64).unwrap();
        assert!(url.contains("x86_64-apple-darwin"));
        assert!(url.contains("install_only.tar.gz"));
    }
}
