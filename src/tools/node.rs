use crate::error::Result;
use crate::tools::{Arch, Tool, Version};
use serde::Deserialize;

pub struct NodeTool;

#[derive(Deserialize, Debug)]
struct NodeRelease {
    version: String,
    #[allow(dead_code)]
    date: String,
    #[allow(dead_code)]
    files: Vec<String>,
    lts: serde_json::Value,
}

impl Tool for NodeTool {
    fn name(&self) -> &str {
        "node"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        let url = "https://nodejs.org/dist/index.json";
        let response = reqwest::blocking::get(url)?;
        let releases: Vec<NodeRelease> = response.json()?;

        let versions = releases
            .into_iter()
            .map(|r| Version {
                version: r.version.clone(),
                lts: match r.lts {
                    serde_json::Value::String(s) => Some(s),
                    _ => None,
                },
            })
            .collect();

        Ok(versions)
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        // 确保版本号有 v 前缀
        let version = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        let arch_str = match arch {
            Arch::Arm64 => "arm64",
            Arch::X86_64 => "x64",
        };

        Ok(format!(
            "https://nodejs.org/dist/{}/node-{}-darwin-{}.tar.gz",
            version, version, arch_str
        ))
    }

    fn checksum_url(&self, version: &str, _arch: Arch) -> Option<String> {
        let version = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        Some(format!(
            "https://nodejs.org/dist/{}/SHASUMS256.txt",
            version
        ))
    }

    fn bin_names(&self) -> Vec<&str> {
        vec!["node", "npm", "npx"]
    }

    fn bin_subpath(&self) -> &str {
        "bin"
    }

    fn get_checksum(&self, version: &str, arch: Arch) -> Result<Option<String>> {
        let checksum_url = match self.checksum_url(version, arch) {
            Some(url) => url,
            None => return Ok(None),
        };

        let response = reqwest::blocking::get(&checksum_url)?;
        let content = response.text()?;

        let version = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        let arch_str = match arch {
            Arch::Arm64 => "arm64",
            Arch::X86_64 => "x64",
        };

        let filename = format!("node-{}-darwin-{}.tar.gz", version, arch_str);

        for line in content.lines() {
            if line.contains(&filename) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Ok(Some(parts[0].to_string()));
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
        let tool = NodeTool;
        assert_eq!(tool.name(), "node");
    }

    #[test]
    fn test_bin_names() {
        let tool = NodeTool;
        assert_eq!(tool.bin_names(), vec!["node", "npm", "npx"]);
    }

    #[test]
    fn test_bin_subpath() {
        let tool = NodeTool;
        assert_eq!(tool.bin_subpath(), "bin");
    }

    #[test]
    fn test_bin_paths_default() {
        let tool = NodeTool;
        let paths = tool.bin_paths();
        assert_eq!(
            paths,
            vec![("node", "bin"), ("npm", "bin"), ("npx", "bin"),]
        );
    }

    #[test]
    fn test_download_url_arm64() {
        let tool = NodeTool;
        let url = tool.download_url("20.11.0", Arch::Arm64).unwrap();
        assert_eq!(
            url,
            "https://nodejs.org/dist/v20.11.0/node-v20.11.0-darwin-arm64.tar.gz"
        );
    }

    #[test]
    fn test_download_url_x86() {
        let tool = NodeTool;
        let url = tool.download_url("20.11.0", Arch::X86_64).unwrap();
        assert_eq!(
            url,
            "https://nodejs.org/dist/v20.11.0/node-v20.11.0-darwin-x64.tar.gz"
        );
    }

    #[test]
    fn test_download_url_with_v_prefix() {
        let tool = NodeTool;
        let url = tool.download_url("v20.11.0", Arch::Arm64).unwrap();
        assert_eq!(
            url,
            "https://nodejs.org/dist/v20.11.0/node-v20.11.0-darwin-arm64.tar.gz"
        );
    }

    #[test]
    fn test_checksum_url() {
        let tool = NodeTool;
        let url = tool.checksum_url("20.11.0", Arch::Arm64);
        assert_eq!(
            url,
            Some("https://nodejs.org/dist/v20.11.0/SHASUMS256.txt".to_string())
        );
    }

    #[test]
    fn test_checksum_url_with_v_prefix() {
        let tool = NodeTool;
        let url = tool.checksum_url("v20.11.0", Arch::Arm64);
        assert_eq!(
            url,
            Some("https://nodejs.org/dist/v20.11.0/SHASUMS256.txt".to_string())
        );
    }

    #[test]
    #[ignore] // 需要网络
    fn test_list_remote() {
        let tool = NodeTool;
        let versions = tool.list_remote().unwrap();
        assert!(!versions.is_empty());
        // 第一个版本应该有 v 前缀
        assert!(versions[0].version.starts_with('v'));
    }

    #[test]
    #[ignore]
    fn test_list_remote_has_lts() {
        let tool = NodeTool;
        let versions = tool.list_remote().unwrap();
        // 应该至少有一个 LTS 版本
        let has_lts = versions.iter().any(|v| v.lts.is_some());
        assert!(has_lts);
    }
}
