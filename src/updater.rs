//! Self-update module
//!
//! Fetches the latest vex release from GitHub and replaces the current binary.

use crate::downloader::download_with_retry;
use crate::error::{Result, VexError};
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const GITHUB_API_LATEST: &str = "https://api.github.com/repos/imnotnoahhh/vex/releases/latest";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

/// Detect the current platform asset name suffix.
/// vex releases follow the pattern: vex-{arch}-apple-darwin
fn asset_name() -> Option<&'static str> {
    #[cfg(target_arch = "aarch64")]
    return Some("aarch64-apple-darwin");
    #[cfg(target_arch = "x86_64")]
    return Some("x86_64-apple-darwin");
    #[allow(unreachable_code)]
    None
}

/// Fetch the latest release info from GitHub API.
fn fetch_latest_release() -> Result<GithubRelease> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("vex/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(VexError::Network)?;

    let release: GithubRelease = client
        .get(GITHUB_API_LATEST)
        .send()
        .map_err(VexError::Network)?
        .json()
        .map_err(VexError::Network)?;

    Ok(release)
}

/// Strip leading 'v' from a version tag like "v0.1.7" → "0.1.7".
fn strip_v(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

/// Compare two semver strings. Returns true if `remote` is newer than `local`.
fn is_newer(local: &str, remote: &str) -> bool {
    let parse = |s: &str| -> (u64, u64, u64) {
        let parts: Vec<u64> = s.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(remote) > parse(local)
}

/// Run the self-update flow.
pub fn self_update() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    println!("Checking for updates...");

    let release = fetch_latest_release()?;
    let latest_version = strip_v(&release.tag_name);

    if !is_newer(current_version, latest_version) {
        println!(
            "{} vex {} is already up to date.",
            "✓".green(),
            current_version.cyan()
        );
        return Ok(());
    }

    println!(
        "New version available: {} → {}",
        current_version.dimmed(),
        latest_version.green().bold()
    );

    let arch_suffix = asset_name()
        .ok_or_else(|| VexError::Parse("Unsupported architecture for self-update".to_string()))?;

    // Find the matching asset — prefer archive formats, then bare binary
    // Exclude .sha256 checksum files explicitly
    let asset = release
        .assets
        .iter()
        .find(|a| {
            a.name.contains(arch_suffix)
                && a.name.ends_with(".tar.xz")
                && !a.name.ends_with(".sha256")
        })
        .or_else(|| {
            release.assets.iter().find(|a| {
                a.name.contains(arch_suffix)
                    && a.name.ends_with(".tar.gz")
                    && !a.name.ends_with(".sha256")
            })
        })
        .or_else(|| {
            // Fallback: bare binary (no known archive extension)
            release.assets.iter().find(|a| {
                a.name.contains(arch_suffix)
                    && !a.name.contains(".tar.gz")
                    && !a.name.contains(".tar.xz")
                    && !a.name.contains(".zip")
                    && !a.name.ends_with(".sha256")
            })
        })
        .ok_or_else(|| {
            VexError::Parse(format!(
                "No release asset found for platform: {}",
                arch_suffix
            ))
        })?;

    // Determine current binary path
    let current_exe = std::env::current_exe().map_err(VexError::Io)?;

    // Download to a temp file next to the binary
    let tmp_path = current_exe.with_extension("tmp");
    println!("Downloading {}...", asset.name);
    download_with_retry(&asset.browser_download_url, &tmp_path, 3)?;

    // If it's an archive, extract the binary from it
    let final_tmp = if asset.name.ends_with(".tar.xz") {
        extract_binary_from_tarball_xz(&tmp_path, &current_exe)?
    } else if asset.name.ends_with(".tar.gz") {
        extract_binary_from_tarball(&tmp_path, &current_exe)?
    } else {
        tmp_path.clone()
    };

    // Make executable
    fs::set_permissions(&final_tmp, fs::Permissions::from_mode(0o755))?;

    // Atomic replace: rename tmp → current binary
    fs::rename(&final_tmp, &current_exe)?;

    // Clean up tmp if it still exists (e.g. tarball case left it)
    if tmp_path.exists() && tmp_path != final_tmp {
        let _ = fs::remove_file(&tmp_path);
    }

    println!(
        "{} Updated to vex {}",
        "✓".green(),
        latest_version.cyan().bold()
    );

    Ok(())
}

/// Extract the `vex` binary from a tar.gz archive, writing it to a temp path.
fn extract_binary_from_tarball(tarball: &Path, current_exe: &Path) -> Result<PathBuf> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let file = fs::File::open(tarball)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    let out_path = current_exe.with_extension("extracted_tmp");

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if file_name == "vex" {
            entry.unpack(&out_path)?;
            return Ok(out_path);
        }
    }

    Err(VexError::Parse(
        "Could not find 'vex' binary inside the release archive".to_string(),
    ))
}

/// Extract the `vex` binary from a tar.xz archive, writing it to a temp path.
fn extract_binary_from_tarball_xz(tarball: &Path, current_exe: &Path) -> Result<PathBuf> {
    use tar::Archive;
    use xz2::read::XzDecoder;

    let file = fs::File::open(tarball)?;
    let xz = XzDecoder::new(file);
    let mut archive = Archive::new(xz);

    let out_path = current_exe.with_extension("extracted_tmp");

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if file_name == "vex" {
            entry.unpack(&out_path)?;
            return Ok(out_path);
        }
    }

    Err(VexError::Parse(
        "Could not find 'vex' binary inside the release archive".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_v() {
        assert_eq!(strip_v("v0.1.7"), "0.1.7");
        assert_eq!(strip_v("0.1.7"), "0.1.7");
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.1.6", "0.1.7"));
        assert!(is_newer("0.1.6", "0.2.0"));
        assert!(is_newer("0.1.6", "1.0.0"));
        assert!(!is_newer("0.1.7", "0.1.7"));
        assert!(!is_newer("0.1.7", "0.1.6"));
    }

    #[test]
    fn test_asset_name_is_some() {
        assert!(asset_name().is_some());
    }

    #[test]
    fn test_extract_binary_from_tarball_found() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let tarball = dir.path().join("release.tar.gz");
        let fake_exe = dir.path().join("fake_vex");

        // Build a tar.gz containing a file named "vex"
        let gz = fs::File::create(&tarball).unwrap();
        let enc = GzEncoder::new(gz, Compression::default());
        let mut ar = tar::Builder::new(enc);
        let content = b"#!/bin/sh\necho vex";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        ar.append_data(
            &mut header,
            "vex-0.1.7-aarch64-apple-darwin/vex",
            &content[..],
        )
        .unwrap();
        ar.into_inner().unwrap().finish().unwrap();

        let result = extract_binary_from_tarball(&tarball, &fake_exe);
        assert!(result.is_ok(), "extract failed: {:?}", result.err());
        let out = result.unwrap();
        assert!(out.exists());
    }

    #[test]
    fn test_extract_binary_from_tarball_not_found() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let tarball = dir.path().join("empty.tar.gz");
        let fake_exe = dir.path().join("fake_vex");

        // Build a tar.gz with no "vex" entry
        let gz = fs::File::create(&tarball).unwrap();
        let enc = GzEncoder::new(gz, Compression::default());
        let mut ar = tar::Builder::new(enc);
        ar.finish().unwrap();

        let result = extract_binary_from_tarball(&tarball, &fake_exe);
        assert!(result.is_err());
    }
}
