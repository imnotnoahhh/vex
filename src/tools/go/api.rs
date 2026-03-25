use super::dist::{arch_suffix, ensure_go_prefix, strip_go_prefix};
use crate::error::Result;
use crate::http;
use crate::tools::{Arch, Version};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(super) struct GoRelease {
    version: String,
    stable: bool,
    files: Vec<GoFile>,
}

#[derive(Deserialize, Debug)]
struct GoFile {
    os: String,
    arch: String,
    sha256: String,
    kind: String,
}

pub(super) fn fetch_releases() -> Result<Vec<GoRelease>> {
    http::get_json_in_current_context(
        "https://go.dev/dl/?mode=json",
        concat!("vex/", env!("CARGO_PKG_VERSION")),
    )
}

pub(super) fn release_versions(releases: Vec<GoRelease>) -> Vec<Version> {
    releases
        .into_iter()
        .filter(|release| release.stable)
        .map(|release| Version {
            version: strip_go_prefix(&release.version),
            lts: None,
        })
        .collect()
}

pub(super) fn checksum_for_release(
    releases: Vec<GoRelease>,
    version: &str,
    arch: Arch,
) -> Option<String> {
    let go_version = ensure_go_prefix(version);
    releases
        .into_iter()
        .find(|release| release.version == go_version)
        .and_then(|release| find_release_checksum(release, arch))
}

pub(super) fn resolve_alias_from_versions(versions: &[Version], alias: &str) -> Option<String> {
    match alias {
        "latest" => versions.first().map(|version| version.version.clone()),
        _ => {
            let parts: Vec<&str> = alias.split('.').collect();
            if parts.len() != 2 {
                return None;
            }
            let major = parts[0];
            let minor = parts[1];
            let prefix = if minor == "x" {
                format!("{}.", major)
            } else {
                format!("{}.{}.", major, minor)
            };
            versions
                .iter()
                .find(|version| version.version.starts_with(&prefix))
                .map(|version| version.version.clone())
        }
    }
}

fn find_release_checksum(release: GoRelease, arch: Arch) -> Option<String> {
    let target_arch = arch_suffix(arch);
    release.files.into_iter().find_map(|file| {
        (file.os == "darwin" && file.arch == target_arch && file.kind == "archive")
            .then_some(file.sha256)
    })
}
