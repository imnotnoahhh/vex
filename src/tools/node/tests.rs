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
    assert_eq!(paths, vec![("node", "bin"), ("npm", "bin"), ("npx", "bin")]);
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
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_list_remote() {
    let tool = NodeTool;
    let versions = tool.list_remote().unwrap();
    assert!(!versions.is_empty());
    assert!(versions[0].version.starts_with('v'));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_list_remote_has_lts() {
    let tool = NodeTool;
    let versions = tool.list_remote().unwrap();
    assert!(versions.iter().any(|v| v.lts.is_some()));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_latest() {
    let result = NodeTool.resolve_alias("latest").unwrap();
    assert!(result.is_some());
    assert!(!result.as_ref().unwrap().starts_with('v'));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_lts() {
    let result = NodeTool.resolve_alias("lts").unwrap();
    assert!(result.is_some());
    assert!(!result.as_ref().unwrap().starts_with('v'));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_lts_codename() {
    let result = NodeTool.resolve_alias("lts-iron").unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().starts_with("20."));
}

#[test]
fn test_resolve_alias_unknown() {
    let result = NodeTool.resolve_alias("foobar").unwrap();
    assert!(result.is_none());
}
