//! Python tool implementation
//!
//! Uses python-build-standalone (astral-sh/python-build-standalone) GitHub releases
//! to provide prebuilt CPython binaries. Supports version aliases based on Python's
//! support lifecycle (bugfix, security, end-of-life).

use crate::error::Result;
use crate::error::VexError;
use crate::http;
use crate::tools::{Arch, Tool, Version};
use reqwest::blocking::Client;
use std::collections::{BTreeMap, BTreeSet};
use tracing::warn;

/// Python tool (python-build-standalone prebuilt CPython)
pub struct PythonTool;

const PYTHON_STATUS_URL: &str = "https://devguide.python.org/versions/";

/// Python support status based on lifecycle
/// See: <https://devguide.python.org/versions/>
#[derive(Debug, Clone, PartialEq)]
pub enum SupportStatus {
    Feature,
    Bugfix,
    Security,
    EndOfLife,
}

impl SupportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportStatus::Feature => "feature",
            SupportStatus::Bugfix => "bugfix",
            SupportStatus::Security => "security",
            SupportStatus::EndOfLife => "end-of-life",
        }
    }

    /// Determine support status from major.minor version string
    pub fn from_version(major_minor: &str) -> Self {
        match major_minor {
            "3.15" => SupportStatus::Feature,
            "3.14" | "3.13" => SupportStatus::Bugfix,
            "3.12" | "3.11" | "3.10" => SupportStatus::Security,
            _ => SupportStatus::EndOfLife,
        }
    }
}

fn create_github_client() -> Result<Client> {
    http::client_for_current_context("vex-version-manager")
}

fn fetch_text_with_retry(client: &Client, url: &str) -> Result<String> {
    let settings = crate::config::load_effective_settings_for_current_dir()?;
    let mut attempts = 0;
    let max_attempts = settings.network.download_retries.max(1);

    loop {
        match client.get(url).send() {
            Ok(response) => match response.error_for_status() {
                Ok(ok_response) => match ok_response.text() {
                    Ok(text) => return Ok(text),
                    Err(err) => {
                        if attempts + 1 < max_attempts {
                            warn!(
                                "Python upstream text fetch failed (attempt {}/{}): {}",
                                attempts + 1,
                                max_attempts,
                                err
                            );
                            attempts += 1;
                            std::thread::sleep(settings.network.retry_base_delay);
                            continue;
                        }
                        return Err(VexError::Network(err));
                    }
                },
                Err(err) => {
                    if err.status().map(|s| s.is_client_error()).unwrap_or(false) {
                        return Err(VexError::Network(err));
                    }
                    if attempts + 1 < max_attempts {
                        warn!(
                            "Python upstream request failed (attempt {}/{}): {}",
                            attempts + 1,
                            max_attempts,
                            err
                        );
                        attempts += 1;
                        std::thread::sleep(settings.network.retry_base_delay);
                        continue;
                    }
                    return Err(VexError::Network(err));
                }
            },
            Err(err) => {
                if attempts + 1 < max_attempts {
                    warn!(
                        "Python upstream request failed (attempt {}/{}): {}",
                        attempts + 1,
                        max_attempts,
                        err
                    );
                    attempts += 1;
                    std::thread::sleep(settings.network.retry_base_delay);
                    continue;
                }
                return Err(VexError::Network(err));
            }
        }
    }
}

/// Resolve the latest python-build-standalone release tag without downloading the giant JSON payload.
fn fetch_latest_release_tag() -> Result<String> {
    let url = "https://github.com/astral-sh/python-build-standalone/releases/latest";
    let client = create_github_client()?;
    let response = client.get(url).send()?.error_for_status()?;
    let final_url = response.url().clone();
    let tag = final_url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .filter(|segment| !segment.is_empty())
        .ok_or_else(|| {
            VexError::Parse("Unable to determine python-build-standalone release tag".to_string())
        })?;
    Ok(tag.to_string())
}

fn fetch_sha256sums(tag: &str) -> Result<String> {
    let client = create_github_client()?;
    let sha256_url = format!(
        "https://github.com/astral-sh/python-build-standalone/releases/download/{}/SHA256SUMS",
        tag
    );
    fetch_text_with_retry(&client, &sha256_url)
}

fn fetch_python_lifecycle_statuses() -> Result<BTreeMap<String, String>> {
    let client = create_github_client()?;
    let html = fetch_text_with_retry(&client, PYTHON_STATUS_URL)?;
    let statuses = parse_python_lifecycle_statuses(&html);
    if statuses.is_empty() {
        return Err(VexError::Parse(
            "Unable to parse Python lifecycle statuses from the official version page".to_string(),
        ));
    }
    Ok(statuses)
}

fn fallback_python_lifecycle_statuses() -> BTreeMap<String, String> {
    let mut statuses = BTreeMap::new();
    for minor in ["3.15", "3.14", "3.13", "3.12", "3.11", "3.10"] {
        statuses.insert(
            minor.to_string(),
            SupportStatus::from_version(minor).as_str().to_string(),
        );
    }
    statuses
}

fn strip_html_tags(text: &str) -> String {
    let mut out = String::new();
    let mut inside_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ").trim().to_string()
}

fn parse_table_cells(row_html: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut remaining = row_html;
    while let Some(td_start) = remaining.find("<td") {
        remaining = &remaining[td_start..];
        let Some(cell_start) = remaining.find('>') else {
            break;
        };
        remaining = &remaining[cell_start + 1..];
        let Some(cell_end) = remaining.find("</td>") else {
            break;
        };
        cells.push(strip_html_tags(&remaining[..cell_end]));
        remaining = &remaining[cell_end + "</td>".len()..];
    }
    cells
}

fn parse_python_lifecycle_statuses(html: &str) -> BTreeMap<String, String> {
    let mut statuses = BTreeMap::new();
    let mut remaining = html;
    let mut seen_supported_rows = false;

    while let Some(tr_start) = remaining.find("<tr") {
        remaining = &remaining[tr_start..];
        let Some(row_end) = remaining.find("</tr>") else {
            break;
        };
        let row_html = &remaining[..row_end];
        let cells = parse_table_cells(row_html);
        if cells.len() >= 3 {
            let branch = cells[0].trim();
            let status = cells[2].trim().to_lowercase();
            if branch.starts_with("3.") {
                statuses.insert(branch.to_string(), status);
                seen_supported_rows = true;
            } else if seen_supported_rows && branch.is_empty() {
                break;
            }
        }
        remaining = &remaining[row_end + "</tr>".len()..];
    }

    statuses
}

fn asset_filename(version: &str, tag: &str, arch: Arch) -> String {
    let arch_str = match arch {
        Arch::Arm64 => "aarch64-apple-darwin",
        Arch::X86_64 => "x86_64-apple-darwin",
    };
    format!(
        "cpython-{}+{}-{}-install_only.tar.gz",
        version, tag, arch_str
    )
}

fn find_matching_checksum(content: &str, filename: &str) -> Option<String> {
    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(2, "  ").collect();
        if parts.len() == 2 && parts[1] == filename {
            return Some(parts[0].to_string());
        }
    }
    None
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
        let tag = fetch_latest_release_tag()?;
        let content = fetch_sha256sums(&tag)?;
        let lifecycle_statuses = match fetch_python_lifecycle_statuses() {
            Ok(statuses) => statuses,
            Err(err) => {
                warn!(
                    "Falling back to built-in Python lifecycle statuses after official fetch failed: {}",
                    err
                );
                fallback_python_lifecycle_statuses()
            }
        };

        let mut versions = BTreeSet::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.splitn(2, "  ").collect();
            if parts.len() != 2 {
                continue;
            }
            let filename = parts[1];
            if filename.contains("apple-darwin")
                && filename.ends_with("install_only.tar.gz")
                && !filename.contains("stripped")
            {
                if let Some(ver) = extract_python_version(filename) {
                    versions.insert(ver);
                }
            }
        }

        // Sort descending by version
        let mut versions: Vec<String> = versions.into_iter().collect();
        versions.sort_by(|a, b| {
            let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
            let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
            b_parts.cmp(&a_parts)
        });

        let result = versions
            .into_iter()
            .map(|ver| {
                let mm = get_major_minor(&ver);
                let status = lifecycle_statuses
                    .get(&mm)
                    .cloned()
                    .unwrap_or_else(|| SupportStatus::from_version(&mm).as_str().to_string());
                Version {
                    version: ver,
                    lts: Some(status),
                }
            })
            .collect();

        Ok(result)
    }

    fn download_url(&self, version: &str, arch: Arch) -> Result<String> {
        let tag = fetch_latest_release_tag()?;
        let filename = asset_filename(version, &tag, arch);
        let content = fetch_sha256sums(&tag)?;

        if find_matching_checksum(&content, &filename).is_some() {
            return Ok(format!(
                "https://github.com/astral-sh/python-build-standalone/releases/download/{}/{}",
                tag, filename
            ));
        }

        Err(VexError::VersionNotFound {
            tool: "python".to_string(),
            version: version.to_string(),
            suggestions: String::new(),
        })
    }

    fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
        // SHA256SUMS is a single file for all assets in the release
        // We'll handle it in get_checksum
        None
    }

    fn get_checksum(&self, version: &str, arch: Arch) -> Result<Option<String>> {
        let tag = fetch_latest_release_tag()?;
        let content = fetch_sha256sums(&tag)?;
        let filename = asset_filename(version, &tag, arch);
        Ok(find_matching_checksum(&content, &filename))
    }

    fn resolve_alias(&self, alias: &str) -> Result<Option<String>> {
        let versions = self.list_remote()?;

        match alias {
            "latest" | "stable" | "bugfix" => {
                // Return the latest bugfix-phase version
                Ok(versions
                    .iter()
                    .find(|v| v.lts.as_deref() == Some("bugfix"))
                    .map(|v| v.version.clone()))
            }
            "security" => Ok(versions
                .iter()
                .find(|v| v.lts.as_deref() == Some("security"))
                .map(|v| v.version.clone())),
            _ => Ok(None),
        }
    }

    fn bin_names(&self) -> Vec<&str> {
        vec![
            "python3",
            "pip3",
            "python",
            "pip",
            "2to3",
            "idle3",
            "pydoc3",
            "python3-config",
        ]
    }

    fn bin_subpath(&self) -> &str {
        "bin"
    }

    /// After extraction, replace empty placeholder files with symlinks to the
    /// versioned binaries (e.g. python3 → python3.12).
    /// python-build-standalone's install_only tarball ships python3, python,
    /// 2to3, idle3, pydoc3, python3-config as zero-byte placeholders.
    fn post_install(&self, install_dir: &std::path::Path, _arch: Arch) -> Result<()> {
        use std::fs;

        let bin_dir = install_dir.join("bin");

        // Find the versioned python binary (e.g. python3.12)
        let versioned = fs::read_dir(&bin_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .find(|name| {
                name.starts_with("python3.")
                    && name
                        .chars()
                        .last()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
            });

        let versioned = match versioned {
            Some(v) => v,
            None => return Ok(()), // nothing to fix
        };

        // e.g. "3.12" from "python3.12"
        let minor = versioned.trim_start_matches("python");

        // Map: placeholder name → versioned target
        let replacements = [
            ("python3", versioned.as_str()),
            ("python", versioned.as_str()),
            ("2to3", &format!("2to3-{}", minor) as &str),
            ("idle3", &format!("idle{}", minor) as &str),
            ("pydoc3", &format!("pydoc{}", minor) as &str),
            ("python3-config", &format!("python{}-config", minor) as &str),
        ];

        for (placeholder, target) in &replacements {
            let placeholder_path = bin_dir.join(placeholder);
            let target_path = bin_dir.join(target);

            // Only replace if placeholder is empty and target exists and is non-empty
            if placeholder_path.exists()
                && target_path.exists()
                && fs::metadata(&placeholder_path)?.len() == 0
                && fs::metadata(&target_path)?.len() > 0
            {
                fs::remove_file(&placeholder_path)?;
                std::os::unix::fs::symlink(target, &placeholder_path)?;
            }
        }

        Ok(())
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
        assert!(bins.contains(&"python"));
        assert!(bins.contains(&"pip"));
        assert!(bins.contains(&"2to3"));
        assert!(bins.contains(&"idle3"));
        assert!(bins.contains(&"pydoc3"));
        assert!(bins.contains(&"python3-config"));
        assert_eq!(bins.len(), 8);
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
        assert!(paths.contains(&("python", "bin")));
        assert!(paths.contains(&("pip", "bin")));
        assert!(paths.contains(&("2to3", "bin")));
        assert!(paths.contains(&("idle3", "bin")));
        assert!(paths.contains(&("pydoc3", "bin")));
        assert!(paths.contains(&("python3-config", "bin")));
        assert_eq!(paths.len(), 8);
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
        assert_eq!(SupportStatus::from_version("3.15"), SupportStatus::Feature);
        assert_eq!(SupportStatus::from_version("3.14"), SupportStatus::Bugfix);
        assert_eq!(SupportStatus::from_version("3.13"), SupportStatus::Bugfix);
        assert_eq!(SupportStatus::from_version("3.12"), SupportStatus::Security);
        assert_eq!(SupportStatus::from_version("3.11"), SupportStatus::Security);
        assert_eq!(SupportStatus::from_version("3.10"), SupportStatus::Security);
        assert_eq!(SupportStatus::from_version("3.9"), SupportStatus::EndOfLife);
    }

    #[test]
    fn test_support_status_as_str() {
        assert_eq!(SupportStatus::Feature.as_str(), "feature");
        assert_eq!(SupportStatus::Bugfix.as_str(), "bugfix");
        assert_eq!(SupportStatus::Security.as_str(), "security");
        assert_eq!(SupportStatus::EndOfLife.as_str(), "end-of-life");
    }

    #[test]
    fn test_checksum_url_is_none() {
        // checksum_url returns None; checksums are fetched via get_checksum
        assert_eq!(PythonTool.checksum_url("3.12.13", Arch::Arm64), None);
    }

    #[test]
    fn test_get_major_minor_single_segment() {
        // Fallback: version with no dot returns as-is
        assert_eq!(get_major_minor("3"), "3");
    }

    #[test]
    fn test_extract_python_version_no_plus() {
        // No '+' separator → split returns the whole string after prefix
        let result = extract_python_version("cpython-3.12.13-aarch64.tar.gz");
        assert_eq!(result, Some("3.12.13-aarch64.tar.gz".to_string()));
    }

    #[test]
    fn test_extract_python_version_empty_version() {
        // '+' immediately after prefix → empty string version
        let result = extract_python_version("cpython-+20260303-aarch64.tar.gz");
        assert_eq!(result, Some("".to_string()));
    }

    #[test]
    fn test_support_status_end_of_life_variants() {
        for v in &["3.8", "3.7", "3.6", "2.7"] {
            assert_eq!(SupportStatus::from_version(v), SupportStatus::EndOfLife);
        }
    }

    #[test]
    fn test_support_status_as_str_all_variants() {
        assert_eq!(SupportStatus::Feature.as_str(), "feature");
        assert_eq!(SupportStatus::Bugfix.as_str(), "bugfix");
        assert_eq!(SupportStatus::Security.as_str(), "security");
        assert_eq!(SupportStatus::EndOfLife.as_str(), "end-of-life");
    }

    #[test]
    fn test_parse_python_lifecycle_statuses() {
        let html = r#"
<tbody>
<tr class="row-odd"><td><p>main</p></td><td><p>PEP</p></td><td><p>feature</p></td></tr>
<tr class="row-even"><td><p>3.14</p></td><td><p>PEP 745</p></td><td><p>bugfix</p></td></tr>
<tr class="row-odd"><td><p>3.13</p></td><td><p>PEP 719</p></td><td><p>bugfix</p></td></tr>
<tr class="row-even"><td><p>3.12</p></td><td><p>PEP 693</p></td><td><p>security</p></td></tr>
</tbody>
"#;
        let statuses = parse_python_lifecycle_statuses(html);
        assert_eq!(statuses.get("3.14").map(String::as_str), Some("bugfix"));
        assert_eq!(statuses.get("3.13").map(String::as_str), Some("bugfix"));
        assert_eq!(statuses.get("3.12").map(String::as_str), Some("security"));
        assert_eq!(statuses.get("main"), None);
    }

    #[test]
    fn test_asset_filename_arm64() {
        let filename = asset_filename("3.14.3", "20260310", Arch::Arm64);
        assert_eq!(
            filename,
            "cpython-3.14.3+20260310-aarch64-apple-darwin-install_only.tar.gz"
        );
    }

    #[test]
    fn test_find_matching_checksum() {
        let content = "\
aaaaaaaa  cpython-3.13.12+20260310-aarch64-apple-darwin-install_only.tar.gz\n\
bbbbbbbb  cpython-3.14.3+20260310-aarch64-apple-darwin-install_only.tar.gz\n";
        let filename = "cpython-3.14.3+20260310-aarch64-apple-darwin-install_only.tar.gz";
        assert_eq!(
            find_matching_checksum(content, filename),
            Some("bbbbbbbb".to_string())
        );
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
