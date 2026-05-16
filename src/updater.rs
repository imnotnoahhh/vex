//! Self-update module
//!
//! Fetches the latest vex release from GitHub and replaces the current binary.

mod extract;
mod release;
mod repair;

use crate::downloader::{download_with_retry, verify_checksum};
use crate::error::{Result, VexError};
use owo_colors::OwoColorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use extract::{extract_binary_from_tarball, extract_binary_from_tarball_xz};
use release::{asset_name, fetch_latest_release, is_newer, select_release_asset, strip_v};
use repair::detect_and_repair_broken_installations;

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

    let asset = select_release_asset(&release, arch_suffix)?;

    // Determine current binary path
    let current_exe = std::env::current_exe().map_err(VexError::Io)?;

    // Download to a temp file next to the binary
    let tmp_path = current_exe.with_extension("tmp");
    println!("Downloading {}...", asset.name);
    download_with_retry(
        &asset.browser_download_url,
        &tmp_path,
        crate::config::download_retries()?,
    )?;

    let checksum_path = current_exe.with_extension("tmp.sha256");
    let checksum_url = format!("{}.sha256", asset.browser_download_url);
    println!("Verifying release checksum...");
    download_with_retry(
        &checksum_url,
        &checksum_path,
        crate::config::download_retries()?,
    )
    .map_err(|err| {
        VexError::Config(format!(
            "Release checksum not available at {}. Refusing self-update without checksum verification. Error: {}",
            checksum_url, err
        ))
    })?;
    let expected_checksum = parse_checksum_sidecar(&fs::read_to_string(&checksum_path)?)?;
    verify_checksum(&tmp_path, &expected_checksum)?;

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
    if checksum_path.exists() {
        let _ = fs::remove_file(&checksum_path);
    }

    println!(
        "{} Updated to vex {}",
        "✓".green(),
        latest_version.cyan().bold()
    );

    // Detect and repair broken installations from old vex versions
    detect_and_repair_broken_installations(current_version)?;

    Ok(())
}

fn parse_checksum_sidecar(content: &str) -> Result<String> {
    let Some(token) = content.split_whitespace().next() else {
        return Err(VexError::Parse(
            "Release checksum file was empty. Refusing self-update.".to_string(),
        ));
    };

    if token.len() != 64 || !token.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(VexError::Parse(format!(
            "Release checksum file did not contain a valid SHA256 digest: {}",
            token
        )));
    }

    Ok(token.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::parse_checksum_sidecar;

    #[test]
    fn test_parse_checksum_sidecar_accepts_sha256sum_format() {
        let checksum = "A".repeat(64);
        let parsed = parse_checksum_sidecar(&format!("{}  vex.tar.gz\n", checksum)).unwrap();
        assert_eq!(parsed, "a".repeat(64));
    }

    #[test]
    fn test_parse_checksum_sidecar_rejects_missing_digest() {
        assert!(parse_checksum_sidecar("").is_err());
        assert!(parse_checksum_sidecar("not-a-checksum vex.tar.gz").is_err());
    }
}
