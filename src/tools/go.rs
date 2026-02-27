use crate::error::Result;
use crate::tools::{Arch, Tool, Version};
use serde::Deserialize;

pub struct GoTool;

#[derive(Deserialize, Debug)]
struct GoRelease {
    version: String,
    stable: bool,
    files: Vec<GoFile>,
}

#[derive(Deserialize, Debug)]
struct GoFile {
    #[allow(dead_code)]
    filename: String,
    os: String,
    arch: String,
    #[allow(dead_code)]
    version: String,
    sha256: String,
    #[allow(dead_code)]
    size: u64,
    kind: String,
}

impl Tool for GoTool {
    fn name(&self) -> &str {
        "go"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        let url = "https://go.dev/dl/?mode=json";
        let response = reqwest::blocking::get(url)?;
        let releases: Vec<GoRelease> = response.json()?;

        let versions = releases
            .into_iter()
            .filter(|r| r.stable) // 只显示稳定版本
            .map(|r| Version {
                // 去掉 "go" 前缀，保持与其他工具一致（如 1.23.5 而非 go1.23.5）
                version: r
                    .version
                    .strip_prefix("go")
                    .unwrap_or(&r.version)
                    .to_string(),
                lts: None, // Go 没有 LTS 概念
            })
            .collect();

        Ok(versions)
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        // 确保版本号有 go 前缀
        let version = if version.starts_with("go") {
            version.to_string()
        } else {
            format!("go{}", version)
        };

        let arch_str = match arch {
            Arch::Arm64 => "arm64",
            Arch::X86_64 => "amd64", // Go 使用 amd64 而不是 x64
        };

        Ok(format!(
            "https://go.dev/dl/{}.darwin-{}.tar.gz",
            version, arch_str
        ))
    }

    fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
        // Go 的 SHA256 直接在 JSON API 中，不需要单独的 checksum URL
        None
    }

    fn bin_names(&self) -> Vec<&str> {
        vec!["go", "gofmt"]
    }

    fn bin_subpath(&self) -> &str {
        "bin"
    }

    fn get_checksum(&self, version: &str, arch: Arch) -> Result<Option<String>> {
        let url = "https://go.dev/dl/?mode=json";
        let response = reqwest::blocking::get(url)?;
        let releases: Vec<GoRelease> = response.json()?;

        let go_version = if version.starts_with("go") {
            version.to_string()
        } else {
            format!("go{}", version)
        };

        let arch_str = match arch {
            Arch::Arm64 => "arm64",
            Arch::X86_64 => "amd64",
        };

        for release in releases {
            if release.version == go_version {
                for file in release.files {
                    if file.os == "darwin" && file.arch == arch_str && file.kind == "archive" {
                        return Ok(Some(file.sha256));
                    }
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(GoTool.name(), "go");
    }

    #[test]
    fn test_bin_names() {
        assert_eq!(GoTool.bin_names(), vec!["go", "gofmt"]);
    }

    #[test]
    fn test_bin_subpath() {
        assert_eq!(GoTool.bin_subpath(), "bin");
    }

    #[test]
    fn test_bin_paths_default() {
        let paths = GoTool.bin_paths();
        assert_eq!(paths, vec![("go", "bin"), ("gofmt", "bin")]);
    }

    #[test]
    fn test_download_url_arm64() {
        let url = GoTool.download_url("1.23.5", Arch::Arm64).unwrap();
        assert_eq!(url, "https://go.dev/dl/go1.23.5.darwin-arm64.tar.gz");
    }

    #[test]
    fn test_download_url_x86() {
        let url = GoTool.download_url("1.23.5", Arch::X86_64).unwrap();
        assert_eq!(url, "https://go.dev/dl/go1.23.5.darwin-amd64.tar.gz");
    }

    #[test]
    fn test_download_url_with_go_prefix() {
        let url = GoTool.download_url("go1.23.5", Arch::Arm64).unwrap();
        assert_eq!(url, "https://go.dev/dl/go1.23.5.darwin-arm64.tar.gz");
    }

    #[test]
    fn test_checksum_url_is_none() {
        assert_eq!(GoTool.checksum_url("1.23.5", Arch::Arm64), None);
    }

    #[test]
    #[ignore] // 需要网络
    fn test_list_remote() {
        let versions = GoTool.list_remote().unwrap();
        assert!(!versions.is_empty());
        // 版本号应该不带 "go" 前缀（如 1.23.5）
        assert!(!versions[0].version.starts_with("go"));
        assert!(versions[0].version.contains('.'));
    }
}
