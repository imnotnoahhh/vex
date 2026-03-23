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
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_list_remote() {
    let versions = GoTool.list_remote().unwrap();
    assert!(!versions.is_empty());
    assert!(!versions[0].version.starts_with("go"));
    assert!(versions[0].version.contains('.'));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_latest() {
    let result = GoTool.resolve_alias("latest").unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().contains('.'));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_minor_version() {
    let result = GoTool.resolve_alias("1.25").unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().starts_with("1.25."));
}

#[test]
fn test_resolve_alias_unknown() {
    let result = GoTool.resolve_alias("foobar").unwrap();
    assert!(result.is_none());

    let result = GoTool.resolve_alias("lts").unwrap();
    assert!(result.is_none());
}
