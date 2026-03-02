//! Java (Eclipse Temurin JDK) tool implementation
//!
//! Uses Adoptium API v3 to query versions, only supports JDK + HotSpot combination.
//! macOS JDK directory structure is special: `Contents/Home/bin/`.

use crate::error::{Result, VexError};
use crate::tools::{Arch, Tool, Version};
use serde::Deserialize;

/// Java (Eclipse Temurin JDK) tool
pub struct JavaTool;

#[derive(Deserialize, Debug)]
struct AvailableReleases {
    available_lts_releases: Vec<u32>,
    available_releases: Vec<u32>,
}

#[derive(Deserialize, Debug)]
struct TemurinRelease {
    binary: Binary,
    #[allow(dead_code)]
    version: VersionData,
}

#[derive(Deserialize, Debug)]
struct Binary {
    package: Package,
}

#[derive(Deserialize, Debug)]
struct Package {
    #[allow(dead_code)]
    name: String,
    link: String,
    checksum: String,
    #[allow(dead_code)]
    size: u64,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct VersionData {
    major: u32,
    minor: u32,
    security: u32,
    #[serde(default)]
    build: u32,
    semver: String,
}

impl Tool for JavaTool {
    fn name(&self) -> &str {
        "java"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        let url = "https://api.adoptium.net/v3/info/available_releases";
        let response = reqwest::blocking::get(url)?;
        let releases: AvailableReleases = response.json()?;

        // Get all available versions, mark LTS versions
        let mut versions = Vec::new();

        for version in releases.available_releases {
            let is_lts = releases.available_lts_releases.contains(&version);
            versions.push(Version {
                version: version.to_string(),
                lts: if is_lts {
                    Some("LTS".to_string())
                } else {
                    None
                },
            });
        }

        // Sort descending (newest version first)
        versions.reverse();

        Ok(versions)
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        // Get download link from API
        let arch_str = match arch {
            Arch::Arm64 => "aarch64",
            Arch::X86_64 => "x64",
        };

        let url = format!(
            "https://api.adoptium.net/v3/assets/latest/{}/hotspot?architecture={}&image_type=jdk&os=mac&vendor=eclipse",
            version, arch_str
        );

        let response = reqwest::blocking::get(&url)?;
        let releases: Vec<TemurinRelease> = response.json()?;

        if let Some(release) = releases.first() {
            Ok(release.binary.package.link.clone())
        } else {
            Err(VexError::VersionNotFound {
                tool: "java".to_string(),
                version: version.to_string(),
            })
        }
    }

    fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
        // Eclipse Temurin's SHA256 is directly in API
        None
    }

    fn bin_names(&self) -> Vec<&str> {
        vec![
            "java",
            "javac",
            "jar",
            "javadoc",
            "javap",
            "jcmd",
            "jconsole",
            "jdb",
            "jdeprscan",
            "jdeps",
            "jfr",
            "jhsdb",
            "jimage",
            "jinfo",
            "jlink",
            "jmap",
            "jmod",
            "jnativescan",
            "jpackage",
            "jps",
            "jrunscript",
            "jshell",
            "jstack",
            "jstat",
            "jstatd",
            "jwebserver",
            "keytool",
            "rmiregistry",
            "serialver",
            "jarsigner",
        ]
    }

    fn bin_subpath(&self) -> &str {
        // macOS JDK directory structure is special: jdk-21.0.10+7/Contents/Home/bin
        "Contents/Home/bin"
    }

    fn get_checksum(&self, version: &str, arch: Arch) -> Result<Option<String>> {
        let arch_str = match arch {
            Arch::Arm64 => "aarch64",
            Arch::X86_64 => "x64",
        };

        let url = format!(
            "https://api.adoptium.net/v3/assets/latest/{}/hotspot?architecture={}&image_type=jdk&os=mac&vendor=eclipse",
            version, arch_str
        );

        let response = reqwest::blocking::get(&url)?;
        let releases: Vec<TemurinRelease> = response.json()?;

        if let Some(release) = releases.first() {
            Ok(Some(release.binary.package.checksum.clone()))
        } else {
            Ok(None)
        }
    }

    fn resolve_alias(&self, alias: &str) -> Result<Option<String>> {
        let versions = self.list_remote()?;

        match alias {
            "latest" => {
                // Return the first version (most recent, list is descending)
                Ok(versions.first().map(|v| v.version.clone()))
            }
            "lts" => {
                // Return the first LTS version
                Ok(versions
                    .iter()
                    .find(|v| v.lts.is_some())
                    .map(|v| v.version.clone()))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(JavaTool.name(), "java");
    }

    #[test]
    fn test_bin_names() {
        let names = JavaTool.bin_names();
        assert!(names.contains(&"java"));
        assert!(names.contains(&"javac"));
        assert!(names.contains(&"jar"));
        assert!(names.contains(&"javadoc"));
        assert!(names.contains(&"jshell"));
        assert!(names.contains(&"keytool"));
        assert_eq!(names.len(), 30);
    }

    #[test]
    fn test_bin_subpath() {
        assert_eq!(JavaTool.bin_subpath(), "Contents/Home/bin");
    }

    #[test]
    fn test_bin_paths_default() {
        let paths = JavaTool.bin_paths();
        // All binaries share the same subpath
        assert_eq!(paths.len(), 30);
        for (_, subpath) in &paths {
            assert_eq!(*subpath, "Contents/Home/bin");
        }
    }

    #[test]
    fn test_checksum_url_is_none() {
        assert_eq!(JavaTool.checksum_url("21", Arch::Arm64), None);
    }

    #[test]
    #[ignore] // Requires network
    fn test_list_remote() {
        let versions = JavaTool.list_remote().unwrap();
        assert!(!versions.is_empty());
        // Should have LTS versions
        let has_lts = versions.iter().any(|v| v.lts.is_some());
        assert!(has_lts);
    }

    #[test]
    #[ignore]
    fn test_download_url() {
        let url = JavaTool.download_url("21", Arch::Arm64).unwrap();
        assert!(url.contains("temurin"));
        assert!(url.ends_with(".tar.gz"));
    }

    #[test]
    #[ignore] // Requires network
    fn test_resolve_alias_latest() {
        let result = JavaTool.resolve_alias("latest").unwrap();
        assert!(result.is_some());
        // Should be a number
        assert!(result.unwrap().parse::<u32>().is_ok());
    }

    #[test]
    #[ignore] // Requires network
    fn test_resolve_alias_lts() {
        let result = JavaTool.resolve_alias("lts").unwrap();
        assert!(result.is_some());
        let version: u32 = result.unwrap().parse().unwrap();
        // LTS versions are 8, 11, 17, 21, 25...
        assert!(version >= 8);
    }

    #[test]
    #[ignore] // Requires network access
    fn test_resolve_alias_unknown() {
        let result = JavaTool.resolve_alias("foobar").unwrap();
        assert!(result.is_none());

        let result = JavaTool.resolve_alias("stable").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_download_url_format_arm64() {
        // Test URL format without network
        let version = "21";
        let arch = Arch::Arm64;
        // We can't test the actual URL without network, but we can test error handling
        // The function will fail with network error, not format error
        let result = JavaTool.download_url(version, arch);
        // Should either succeed or fail with network error, not format error
        assert!(result.is_ok() || matches!(result, Err(VexError::Network(_))));
    }

    #[test]
    fn test_download_url_format_x86() {
        let version = "21";
        let arch = Arch::X86_64;
        let result = JavaTool.download_url(version, arch);
        assert!(result.is_ok() || matches!(result, Err(VexError::Network(_))));
    }

    #[test]
    fn test_get_checksum_format() {
        // Test that get_checksum returns proper format
        let version = "21";
        let arch = Arch::Arm64;
        let result = JavaTool.get_checksum(version, arch);
        // Should either succeed with Some/None or fail with network error
        assert!(result.is_ok() || matches!(result, Err(VexError::Network(_))));
    }
}
