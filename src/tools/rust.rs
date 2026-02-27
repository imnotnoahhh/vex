use crate::error::{Result, VexError};
use crate::tools::{Arch, Tool, Version};
use serde::Deserialize;

pub struct RustTool;

#[derive(Deserialize, Debug)]
struct RustManifest {
    pkg: Packages,
}

#[derive(Deserialize, Debug)]
struct Packages {
    rust: RustPackage,
}

#[derive(Deserialize, Debug)]
struct RustPackage {
    version: String,
    target: std::collections::HashMap<String, TargetInfo>,
}

#[derive(Deserialize, Debug)]
struct TargetInfo {
    #[allow(dead_code)]
    available: bool,
    #[allow(dead_code)]
    url: Option<String>,
    hash: Option<String>,
}

impl Tool for RustTool {
    fn name(&self) -> &str {
        "rust"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        // Rust 只显示稳定版
        let url = "https://static.rust-lang.org/dist/channel-rust-stable.toml";
        let response = reqwest::blocking::get(url)?;
        let content = response.text()?;

        // 解析 TOML
        let manifest: RustManifest = toml::from_str(&content)
            .map_err(|e| VexError::Parse(format!("Failed to parse Rust manifest: {}", e)))?;

        // 提取版本号（格式：1.93.1 (f4f0e5e1e 2026-02-11)）
        let version_str = manifest.pkg.rust.version;
        let version = version_str
            .split_whitespace()
            .next()
            .unwrap_or(&version_str)
            .to_string();

        Ok(vec![Version {
            version,
            lts: None, // Rust 没有 LTS 概念
        }])
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        let target = match arch {
            Arch::Arm64 => "aarch64-apple-darwin",
            Arch::X86_64 => "x86_64-apple-darwin",
        };

        Ok(format!(
            "https://static.rust-lang.org/dist/rust-{}-{}.tar.gz",
            version, target
        ))
    }

    fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
        // Rust 的 SHA256 直接在 TOML 中
        None
    }

    fn bin_names(&self) -> Vec<&str> {
        vec!["rustc", "cargo"]
    }

    fn bin_subpath(&self) -> &str {
        "rustc/bin"
    }

    fn bin_paths(&self) -> Vec<(&str, &str)> {
        // Rust tarball 中 rustc 和 cargo 在不同子目录
        vec![("rustc", "rustc/bin"), ("cargo", "cargo/bin")]
    }

    fn get_checksum(&self, _version: &str, arch: Arch) -> Result<Option<String>> {
        let url = "https://static.rust-lang.org/dist/channel-rust-stable.toml";
        let response = reqwest::blocking::get(url)?;
        let content = response.text()?;

        let manifest: RustManifest = toml::from_str(&content)
            .map_err(|e| VexError::Parse(format!("Failed to parse Rust manifest: {}", e)))?;

        let target = match arch {
            Arch::Arm64 => "aarch64-apple-darwin",
            Arch::X86_64 => "x86_64-apple-darwin",
        };

        if let Some(target_info) = manifest.pkg.rust.target.get(target) {
            if let Some(hash) = &target_info.hash {
                return Ok(Some(hash.clone()));
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
        assert_eq!(RustTool.name(), "rust");
    }

    #[test]
    fn test_bin_names() {
        assert_eq!(RustTool.bin_names(), vec!["rustc", "cargo"]);
    }

    #[test]
    fn test_bin_subpath() {
        assert_eq!(RustTool.bin_subpath(), "rustc/bin");
    }

    #[test]
    fn test_bin_paths_override() {
        let paths = RustTool.bin_paths();
        assert_eq!(paths, vec![("rustc", "rustc/bin"), ("cargo", "cargo/bin"),]);
    }

    #[test]
    fn test_bin_paths_cargo_not_in_rustc_dir() {
        let paths = RustTool.bin_paths();
        let (_, cargo_path) = paths.iter().find(|(name, _)| *name == "cargo").unwrap();
        assert_eq!(*cargo_path, "cargo/bin");
        assert_ne!(*cargo_path, RustTool.bin_subpath());
    }

    #[test]
    fn test_checksum_url_is_none() {
        assert_eq!(RustTool.checksum_url("1.93.1", Arch::Arm64), None);
    }

    #[test]
    fn test_download_url_arm64() {
        let url = RustTool.download_url("1.93.1", Arch::Arm64).unwrap();
        assert_eq!(
            url,
            "https://static.rust-lang.org/dist/rust-1.93.1-aarch64-apple-darwin.tar.gz"
        );
    }

    #[test]
    fn test_download_url_x86() {
        let url = RustTool.download_url("1.93.1", Arch::X86_64).unwrap();
        assert_eq!(
            url,
            "https://static.rust-lang.org/dist/rust-1.93.1-x86_64-apple-darwin.tar.gz"
        );
    }

    #[test]
    #[ignore] // 需要网络
    fn test_list_remote() {
        let versions = RustTool.list_remote().unwrap();
        assert!(!versions.is_empty());
        // Rust 稳定版格式：x.y.z
        assert!(versions[0].version.contains('.'));
    }
}
