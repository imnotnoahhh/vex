use super::api::{available_versions, lts_versions, AvailableReleases};
use super::*;

#[test]
fn test_name() {
    assert_eq!(JavaTool.name(), "java");
}

#[test]
fn test_bin_names() {
    let names = JavaTool.bin_names();
    assert!(names.contains(&"java"));
    assert!(names.contains(&"javac"));
    assert!(names.contains(&"jar"));
    assert!(names.contains(&"javadoc"));
    assert!(names.contains(&"jshell"));
    assert!(names.contains(&"keytool"));
    assert_eq!(names.len(), 30);
}

#[test]
fn test_bin_subpath() {
    assert_eq!(JavaTool.bin_subpath(), "Contents/Home/bin");
}

#[test]
fn test_bin_paths_default() {
    let paths = JavaTool.bin_paths();
    assert_eq!(paths.len(), 30);
    for (_, subpath) in &paths {
        assert_eq!(*subpath, "Contents/Home/bin");
    }
}

#[test]
fn test_checksum_url_is_none() {
    assert_eq!(JavaTool.checksum_url("21", Arch::Arm64), None);
}

#[test]
fn test_lts_versions_falls_back_to_most_recent_lts() {
    let releases = AvailableReleases {
        available_lts_releases: Vec::new(),
        available_releases: vec![25, 24, 21],
        most_recent_lts: Some(25),
    };

    assert_eq!(lts_versions(&releases), vec![25]);
}

#[test]
fn test_lts_versions_ignores_zero_entries() {
    let releases = AvailableReleases {
        available_lts_releases: vec![0, 25, 21],
        available_releases: vec![25, 24, 21],
        most_recent_lts: Some(0),
    };

    assert_eq!(lts_versions(&releases), vec![25, 21]);
}

#[test]
fn test_available_versions_ignores_zero_entries() {
    let releases = AvailableReleases {
        available_lts_releases: vec![25, 21],
        available_releases: vec![0, 25, 24, 21],
        most_recent_lts: Some(25),
    };

    assert_eq!(available_versions(&releases), vec![25, 24, 21]);
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_list_remote() {
    let versions = JavaTool.list_remote().unwrap();
    assert!(!versions.is_empty());
    let has_lts = versions.iter().any(|version| version.lts.is_some());
    assert!(has_lts);
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_download_url() {
    let url = JavaTool.download_url("21", Arch::Arm64).unwrap();
    assert!(url.contains("temurin"));
    assert!(url.ends_with(".tar.gz"));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_latest() {
    let result = JavaTool.resolve_alias("latest").unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().parse::<u32>().is_ok());
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_lts() {
    let result = JavaTool.resolve_alias("lts").unwrap();
    assert!(result.is_some());
    let version: u32 = result.unwrap().parse().unwrap();
    assert!(version >= 8);
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_unknown() {
    let result = JavaTool.resolve_alias("foobar").unwrap();
    assert!(result.is_none());

    let result = JavaTool.resolve_alias("stable").unwrap();
    assert!(result.is_none());
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_download_url_format_arm64() {
    let result = JavaTool.download_url("21", Arch::Arm64);
    assert!(result.is_ok() || matches!(result, Err(VexError::Network(_))));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_download_url_format_x86() {
    let result = JavaTool.download_url("21", Arch::X86_64);
    assert!(result.is_ok() || matches!(result, Err(VexError::Network(_))));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_get_checksum_format() {
    let result = JavaTool.get_checksum("21", Arch::Arm64);
    assert!(result.is_ok());
}
