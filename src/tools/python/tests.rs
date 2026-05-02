use super::lifecycle::{parse_python_lifecycle_statuses, SupportStatus};
use super::releases::{
    asset_filename, extract_python_version, find_matching_checksum, get_major_minor,
};
use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

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
fn test_python_links_dynamic_toolchain_binaries_except_internal_alias() {
    assert!(PythonTool.link_dynamic_binaries());
    assert!(PythonTool.should_link_dynamic_binary("python3.14"));
    assert!(PythonTool.should_link_dynamic_binary("pip3.14"));
    assert!(!PythonTool.should_link_dynamic_binary("\u{1d70b}thon"));
}

#[test]
fn test_base_paths_are_versioned_under_vex_home() {
    let vex = std::path::Path::new("/tmp/vex-home");
    assert_eq!(
        base_env_dir(vex, "3.13.3"),
        vex.join("python").join("base").join("3.13.3")
    );
    assert_eq!(
        base_bin_dir(vex, "3.13.3"),
        vex.join("python").join("base").join("3.13.3").join("bin")
    );
}

#[test]
fn test_ensure_base_environment_creates_missing_base() {
    let temp = TempDir::new().unwrap();
    let vex = temp.path().join(".vex");
    let install = temp.path().join("python-3.13.3");
    let bin = install.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let python = bin.join("python3");
    fs::write(
        &python,
        r#"#!/bin/sh
if [ "$1" = "-m" ] && [ "$2" = "venv" ]; then
  mkdir -p "$3/bin"
  printf '#!/bin/sh\n' > "$3/bin/python"
  printf '#!/bin/sh\n' > "$3/bin/pip"
  chmod +x "$3/bin/python" "$3/bin/pip"
  exit 0
fi
exit 42
"#,
    )
    .unwrap();
    let mut perms = fs::metadata(&python).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&python, perms).unwrap();

    let base = ensure_base_environment(&vex, "3.13.3", &install).unwrap();
    assert_eq!(base, base_env_dir(&vex, "3.13.3"));
    assert!(base.join("bin/python").exists());
    assert!(base.join("bin/pip").exists());
    assert!(is_base_env_healthy(&vex, "3.13.3"));
}

#[test]
fn test_extract_python_version() {
    assert_eq!(
        extract_python_version("cpython-3.12.13+20260303-aarch64-apple-darwin-install_only.tar.gz"),
        Some("3.12.13".to_string())
    );
    assert_eq!(
        extract_python_version("cpython-3.13.2+20250317-aarch64-apple-darwin-install_only.tar.gz"),
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
    assert_eq!(PythonTool.checksum_url("3.12.13", Arch::Arm64), None);
}

#[test]
fn test_get_major_minor_single_segment() {
    assert_eq!(get_major_minor("3"), "3");
}

#[test]
fn test_extract_python_version_no_plus() {
    let result = extract_python_version("cpython-3.12.13-aarch64.tar.gz");
    assert_eq!(result, Some("3.12.13-aarch64.tar.gz".to_string()));
}

#[test]
fn test_extract_python_version_empty_version() {
    let result = extract_python_version("cpython-+20260303-aarch64.tar.gz");
    assert_eq!(result, Some("".to_string()));
}

#[test]
fn test_support_status_end_of_life_variants() {
    for version in &["3.8", "3.7", "3.6", "2.7"] {
        assert_eq!(
            SupportStatus::from_version(version),
            SupportStatus::EndOfLife
        );
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
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_list_remote() {
    let versions = PythonTool.list_remote().unwrap();
    assert!(!versions.is_empty());
    for version in &versions {
        assert!(version.lts.is_some());
    }
    let versions: Vec<&str> = versions
        .iter()
        .map(|version| version.version.as_str())
        .collect();
    assert!(versions[0] >= versions[1]);
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_latest() {
    let result = PythonTool.resolve_alias("latest").unwrap();
    assert!(result.is_some());
    let version = result.unwrap();
    assert!(version.starts_with("3."));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_security() {
    let result = PythonTool.resolve_alias("security").unwrap();
    assert!(result.is_some());
    let version = result.unwrap();
    let major_minor = get_major_minor(&version);
    assert_eq!(
        SupportStatus::from_version(&major_minor),
        SupportStatus::Security
    );
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_download_url_arm64() {
    let url = PythonTool.download_url("3.12.13", Arch::Arm64).unwrap();
    assert!(url.contains("aarch64-apple-darwin"));
    assert!(url.contains("install_only.tar.gz"));
    assert!(url.contains("3.12.13"));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_download_url_x86() {
    let url = PythonTool.download_url("3.12.13", Arch::X86_64).unwrap();
    assert!(url.contains("x86_64-apple-darwin"));
    assert!(url.contains("install_only.tar.gz"));
}
