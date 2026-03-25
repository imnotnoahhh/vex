use super::{GithubAsset, GithubRelease};
use crate::error::{Result, VexError};

pub(in crate::updater) fn asset_name() -> Option<&'static str> {
    if cfg!(target_arch = "aarch64") {
        Some("aarch64-apple-darwin")
    } else if cfg!(target_arch = "x86_64") {
        Some("x86_64-apple-darwin")
    } else {
        None
    }
}

pub(in crate::updater) fn select_release_asset<'a>(
    release: &'a GithubRelease,
    arch_suffix: &str,
) -> Result<&'a GithubAsset> {
    release
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
        })
}
