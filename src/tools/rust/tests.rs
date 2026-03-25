use super::*;
use crate::tools::Tool;

#[test]
fn test_name() {
    assert_eq!(RustTool.name(), "rust");
}

#[test]
fn test_bin_names() {
    let names = RustTool.bin_names();
    assert_eq!(names.len(), 11);
    assert!(names.contains(&"rustc"));
    assert!(names.contains(&"rustdoc"));
    assert!(names.contains(&"rust-lldb"));
    assert!(names.contains(&"cargo"));
    assert!(names.contains(&"clippy-driver"));
    assert!(names.contains(&"rust-analyzer"));
}

#[test]
fn test_bin_subpath() {
    assert_eq!(RustTool.bin_subpath(), "rustc/bin");
}

#[test]
fn test_bin_paths_override() {
    let paths = RustTool.bin_paths();
    assert_eq!(paths.len(), 11);
    assert!(paths.contains(&("rustc", "rustc/bin")));
    assert!(paths.contains(&("rustdoc", "rustc/bin")));
    assert!(paths.contains(&("rust-lldb", "rustc/bin")));
    assert!(paths.contains(&("cargo", "cargo/bin")));
    assert!(paths.contains(&("clippy-driver", "clippy-preview/bin")));
    assert!(paths.contains(&("rust-analyzer", "rust-analyzer-preview/bin")));
}

#[test]
fn test_bin_paths_cargo_not_in_rustc_dir() {
    let paths = RustTool.bin_paths();
    let (_, cargo_path) = paths.iter().find(|(name, _)| *name == "cargo").unwrap();
    assert_eq!(*cargo_path, "cargo/bin");
    assert_ne!(*cargo_path, RustTool.bin_subpath());
}

#[test]
fn test_checksum_url_points_to_sidecar_file() {
    assert_eq!(
        RustTool.checksum_url("1.93.1", Arch::Arm64),
        Some(
            "https://static.rust-lang.org/dist/rust-1.93.1-aarch64-apple-darwin.tar.gz.sha256"
                .to_string()
        )
    );
}

#[test]
fn test_download_url_arm64() {
    let url = RustTool.download_url("1.93.1", Arch::Arm64).unwrap();
    assert_eq!(
        url,
        "https://static.rust-lang.org/dist/rust-1.93.1-aarch64-apple-darwin.tar.gz"
    );
}

#[test]
fn test_download_url_x86() {
    let url = RustTool.download_url("1.93.1", Arch::X86_64).unwrap();
    assert_eq!(
        url,
        "https://static.rust-lang.org/dist/rust-1.93.1-x86_64-apple-darwin.tar.gz"
    );
}

#[test]
fn test_parse_sha256_sidecar_with_filename() {
    let content =
        "6bafa3b5367019c576751741295e06717f8f28c9d0e6631dcb9496cd142a386a  rust-1.93.1-aarch64-apple-darwin.tar.gz\n";
    assert_eq!(
        parse_sha256_sidecar(content),
        Some("6bafa3b5367019c576751741295e06717f8f28c9d0e6631dcb9496cd142a386a".to_string())
    );
}

#[test]
fn test_parse_sha256_sidecar_hash_only() {
    let content = "6bafa3b5367019c576751741295e06717f8f28c9d0e6631dcb9496cd142a386a\n";
    assert_eq!(
        parse_sha256_sidecar(content),
        Some("6bafa3b5367019c576751741295e06717f8f28c9d0e6631dcb9496cd142a386a".to_string())
    );
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_list_remote() {
    let versions = RustTool.list_remote().unwrap();
    assert!(!versions.is_empty());
    assert!(versions[0].version.contains('.'));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_latest() {
    let result = RustTool.resolve_alias("latest").unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().contains('.'));
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_resolve_alias_stable() {
    let result = RustTool.resolve_alias("stable").unwrap();
    assert!(result.is_some());
    let latest = RustTool.resolve_alias("latest").unwrap();
    assert_eq!(result, latest);
}

#[test]
fn test_resolve_alias_unknown() {
    let result = RustTool.resolve_alias("nightly").unwrap();
    assert!(result.is_none());

    let result = RustTool.resolve_alias("beta").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_post_install_creates_symlinks() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let install_dir = temp_dir.path();
    let target = "aarch64-apple-darwin";

    let std_src = install_dir
        .join(format!("rust-std-{}", target))
        .join("lib/rustlib")
        .join(target)
        .join("lib");
    fs::create_dir_all(&std_src).unwrap();
    fs::write(std_src.join("libstd.rlib"), "fake").unwrap();

    let rustc_lib = install_dir.join("rustc/lib/rustlib").join(target);
    fs::create_dir_all(&rustc_lib).unwrap();

    let rustc_lib_root = install_dir.join("rustc/lib");
    fs::create_dir_all(&rustc_lib_root).unwrap();

    for component in &["clippy-preview", "rustfmt-preview", "rust-analyzer-preview"] {
        fs::create_dir_all(install_dir.join(component)).unwrap();
    }

    let result = RustTool.post_install(install_dir, Arch::Arm64);
    assert!(result.is_ok());

    let std_link = install_dir
        .join("rustc/lib/rustlib")
        .join(target)
        .join("lib");
    assert!(std_link.exists());

    for component in &["clippy-preview", "rustfmt-preview", "rust-analyzer-preview"] {
        let lib_link = install_dir.join(component).join("lib");
        assert!(lib_link.exists(), "{} lib link should exist", component);
    }
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_get_checksum_format() {
    let result = RustTool.get_checksum("1.93.1", Arch::Arm64);
    assert!(result.is_ok());
}
