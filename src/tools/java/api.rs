use super::Arch;
use crate::error::Result;
use crate::http;
use serde::Deserialize;

pub(super) const FALLBACK_LTS_VERSIONS: &[u32] = &[25, 21, 17, 11, 8];

#[derive(Deserialize, Debug)]
pub(super) struct AvailableReleases {
    pub(super) available_lts_releases: Vec<u32>,
    pub(super) available_releases: Vec<u32>,
    #[serde(default)]
    pub(super) most_recent_lts: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub(super) struct TemurinRelease {
    pub(super) binary: Binary,
}

#[derive(Deserialize, Debug)]
pub(super) struct Binary {
    pub(super) package: Package,
}

#[derive(Deserialize, Debug)]
pub(super) struct Package {
    pub(super) link: String,
    pub(super) checksum: String,
}

pub(super) fn fetch_available_releases() -> Result<AvailableReleases> {
    http::get_json_in_current_context(
        "https://api.adoptium.net/v3/info/available_releases",
        concat!("vex/", env!("CARGO_PKG_VERSION")),
    )
}

pub(super) fn fetch_temurin_releases(version: &str, arch: Arch) -> Result<Vec<TemurinRelease>> {
    let url = format!(
        "https://api.adoptium.net/v3/assets/latest/{}/hotspot?architecture={}&image_type=jdk&os=mac&vendor=eclipse",
        version,
        temurin_arch(arch)
    );
    http::get_json_in_current_context(&url, concat!("vex/", env!("CARGO_PKG_VERSION")))
}

pub(super) fn available_versions(releases: &AvailableReleases) -> Vec<u32> {
    releases
        .available_releases
        .iter()
        .copied()
        .filter(|version| *version > 0)
        .collect()
}

pub(super) fn lts_versions(releases: &AvailableReleases) -> Vec<u32> {
    let explicit_lts: Vec<u32> = releases
        .available_lts_releases
        .iter()
        .copied()
        .filter(|version| *version > 0)
        .collect();
    if !explicit_lts.is_empty() {
        return explicit_lts;
    }

    releases
        .most_recent_lts
        .filter(|version| *version > 0)
        .map(|version| vec![version])
        .unwrap_or_default()
}

fn temurin_arch(arch: Arch) -> &'static str {
    match arch {
        Arch::Arm64 => "aarch64",
        Arch::X86_64 => "x64",
    }
}
