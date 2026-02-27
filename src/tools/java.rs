use crate::error::{Result, VexError};
use crate::tools::{Arch, Tool, Version};
use serde::Deserialize;

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

        // 获取所有可用版本，LTS 版本标注
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

        // 降序排列（最新版本在前）
        versions.reverse();

        Ok(versions)
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        // 从 API 获取下载链接
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
        // Eclipse Temurin 的 SHA256 直接在 API 中
        None
    }

    fn bin_names(&self) -> Vec<&str> {
        vec!["java", "javac", "jar"]
    }

    fn bin_subpath(&self) -> &str {
        // macOS 的 JDK 目录结构特殊：jdk-21.0.10+7/Contents/Home/bin
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
        assert_eq!(JavaTool.bin_names(), vec!["java", "javac", "jar"]);
    }

    #[test]
    fn test_bin_subpath() {
        assert_eq!(JavaTool.bin_subpath(), "Contents/Home/bin");
    }

    #[test]
    fn test_bin_paths_default() {
        let paths = JavaTool.bin_paths();
        assert_eq!(
            paths,
            vec![
                ("java", "Contents/Home/bin"),
                ("javac", "Contents/Home/bin"),
                ("jar", "Contents/Home/bin"),
            ]
        );
    }

    #[test]
    fn test_checksum_url_is_none() {
        assert_eq!(JavaTool.checksum_url("21", Arch::Arm64), None);
    }

    #[test]
    #[ignore] // 需要网络
    fn test_list_remote() {
        let versions = JavaTool.list_remote().unwrap();
        assert!(!versions.is_empty());
        // 应该有 LTS 版本
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
}
