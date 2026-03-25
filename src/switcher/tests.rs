use super::*;
use crate::tools::go::GoTool;
use crate::tools::node::NodeTool;
use crate::tools::rust::RustTool;
use std::fs;
use std::path::PathBuf;

pub(crate) fn make_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vex_switcher_test_{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_switch_version_not_found() {
    let base = make_temp_dir("not_found");
    let result = switch_version_in(&NodeTool, "99.99.99", &base);
    assert!(result.is_err());
    if let Err(VexError::VersionNotFound {
        tool,
        version,
        suggestions: _,
    }) = result
    {
        assert_eq!(tool, "node");
        assert_eq!(version, "99.99.99");
    } else {
        panic!("Expected VersionNotFound error");
    }
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_creates_current_symlink() {
    let base = make_temp_dir("current_link");
    let tc = base.join("toolchains/node/20.11.0/bin");
    fs::create_dir_all(&tc).unwrap();
    for name in &["node", "npm", "npx"] {
        fs::write(tc.join(name), "fake").unwrap();
    }

    let result = switch_version_in(&NodeTool, "20.11.0", &base);
    assert!(result.is_ok());

    let current_link = base.join("current/node");
    assert!(current_link.exists());
    let target = fs::read_link(&current_link).unwrap();
    assert!(target.ends_with("toolchains/node/20.11.0"));

    assert!(base.join("bin/node").exists());
    assert!(base.join("bin/npm").exists());
    assert!(base.join("bin/npx").exists());

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_go_creates_correct_links() {
    let base = make_temp_dir("go_links");

    let tc = base.join("toolchains/go/1.23.5/bin");
    fs::create_dir_all(&tc).unwrap();
    fs::write(tc.join("go"), "fake").unwrap();
    fs::write(tc.join("gofmt"), "fake").unwrap();

    let result = switch_version_in(&GoTool, "1.23.5", &base);
    assert!(result.is_ok());

    assert!(base.join("bin/go").exists());
    assert!(base.join("bin/gofmt").exists());

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_rust_separate_bin_paths() {
    let base = make_temp_dir("rust_paths");

    let rustc_dir = base.join("toolchains/rust/1.93.1/rustc/bin");
    let cargo_dir = base.join("toolchains/rust/1.93.1/cargo/bin");
    let rustfmt_dir = base.join("toolchains/rust/1.93.1/rustfmt-preview/bin");
    let clippy_dir = base.join("toolchains/rust/1.93.1/clippy-preview/bin");
    let analyzer_dir = base.join("toolchains/rust/1.93.1/rust-analyzer-preview/bin");
    fs::create_dir_all(&rustc_dir).unwrap();
    fs::create_dir_all(&cargo_dir).unwrap();
    fs::create_dir_all(&rustfmt_dir).unwrap();
    fs::create_dir_all(&clippy_dir).unwrap();
    fs::create_dir_all(&analyzer_dir).unwrap();
    fs::write(rustc_dir.join("rustc"), "fake").unwrap();
    fs::write(rustc_dir.join("rustdoc"), "fake").unwrap();
    fs::write(rustc_dir.join("rust-gdb"), "fake").unwrap();
    fs::write(rustc_dir.join("rust-gdbgui"), "fake").unwrap();
    fs::write(rustc_dir.join("rust-lldb"), "fake").unwrap();
    fs::write(cargo_dir.join("cargo"), "fake").unwrap();
    fs::write(rustfmt_dir.join("rustfmt"), "fake").unwrap();
    fs::write(rustfmt_dir.join("cargo-fmt"), "fake").unwrap();
    fs::write(clippy_dir.join("cargo-clippy"), "fake").unwrap();
    fs::write(clippy_dir.join("clippy-driver"), "fake").unwrap();
    fs::write(analyzer_dir.join("rust-analyzer"), "fake").unwrap();

    let result = switch_version_in(&RustTool, "1.93.1", &base);
    assert!(result.is_ok());

    let rustc_target = fs::read_link(base.join("bin/rustc")).unwrap();
    let cargo_target = fs::read_link(base.join("bin/cargo")).unwrap();
    assert!(rustc_target.to_string_lossy().contains("rustc/bin/rustc"));
    assert!(cargo_target.to_string_lossy().contains("cargo/bin/cargo"));

    assert!(base.join("bin/rustfmt").exists());
    assert!(base.join("bin/cargo-fmt").exists());
    assert!(base.join("bin/cargo-clippy").exists());
    assert!(base.join("bin/clippy-driver").exists());
    assert!(base.join("bin/rust-analyzer").exists());
    assert!(base.join("bin/rustdoc").exists());
    assert!(base.join("bin/rust-gdb").exists());
    assert!(base.join("bin/rust-gdbgui").exists());
    assert!(base.join("bin/rust-lldb").exists());

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_replaces_existing_links() {
    let base = make_temp_dir("replace_links");

    let tc_v1 = base.join("toolchains/node/1.0.0/bin");
    fs::create_dir_all(&tc_v1).unwrap();
    for name in &["node", "npm", "npx"] {
        fs::write(tc_v1.join(name), "v1").unwrap();
    }
    switch_version_in(&NodeTool, "1.0.0", &base).unwrap();

    let tc_v2 = base.join("toolchains/node/2.0.0/bin");
    fs::create_dir_all(&tc_v2).unwrap();
    for name in &["node", "npm", "npx"] {
        fs::write(tc_v2.join(name), "v2").unwrap();
    }
    switch_version_in(&NodeTool, "2.0.0", &base).unwrap();

    let target = fs::read_link(base.join("current/node")).unwrap();
    assert!(target.ends_with("toolchains/node/2.0.0"));

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_dynamic_binary_detection() {
    use std::os::unix::fs::PermissionsExt;
    let base = make_temp_dir("dynamic_bin");

    let tc_v24 = base.join("toolchains/node/24.0.0/bin");
    fs::create_dir_all(&tc_v24).unwrap();
    for name in &["node", "npm", "npx", "corepack"] {
        let path = tc_v24.join(name);
        fs::write(&path, "v24").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }
    switch_version_in(&NodeTool, "24.0.0", &base).unwrap();
    assert!(base.join("bin/corepack").exists());

    let tc_v25 = base.join("toolchains/node/25.0.0/bin");
    fs::create_dir_all(&tc_v25).unwrap();
    for name in &["node", "npm", "npx"] {
        let path = tc_v25.join(name);
        fs::write(&path, "v25").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }
    switch_version_in(&NodeTool, "25.0.0", &base).unwrap();

    assert!(!base.join("bin/corepack").exists());
    assert!(base.join("bin/node").exists());
    assert!(base.join("bin/npm").exists());
    assert!(base.join("bin/npx").exists());

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_cleanup_stale_symlinks() {
    use std::os::unix::fs::PermissionsExt;
    let base = make_temp_dir("cleanup_stale");

    let tc_v1 = base.join("toolchains/node/1.0.0/bin");
    fs::create_dir_all(&tc_v1).unwrap();
    for name in &["node", "npm", "npx", "extra-tool"] {
        let path = tc_v1.join(name);
        fs::write(&path, "v1").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }
    switch_version_in(&NodeTool, "1.0.0", &base).unwrap();
    assert!(base.join("bin/extra-tool").exists());

    let tc_v2 = base.join("toolchains/node/2.0.0/bin");
    fs::create_dir_all(&tc_v2).unwrap();
    for name in &["node", "npm", "npx"] {
        let path = tc_v2.join(name);
        fs::write(&path, "v2").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }
    switch_version_in(&NodeTool, "2.0.0", &base).unwrap();

    assert!(!base.join("bin/extra-tool").exists());
    assert!(base.join("bin/node").exists());
    assert!(base.join("bin/npm").exists());
    assert!(base.join("bin/npx").exists());

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_version_empty_toolchain() {
    let base = make_temp_dir("empty_toolchain");
    let tc = base.join("toolchains/node/20.0.0/bin");
    fs::create_dir_all(&tc).unwrap();

    let result = switch_version_in(&NodeTool, "20.0.0", &base);
    assert!(result.is_ok());

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_version_creates_bin_directory() {
    let base = make_temp_dir("create_bin_dir");

    let tc = base.join("toolchains/go/1.21.0/bin");
    fs::create_dir_all(&tc).unwrap();
    fs::write(tc.join("go"), "fake").unwrap();

    let bin_dir = base.join("bin");
    assert!(!bin_dir.exists());

    let result = switch_version_in(&GoTool, "1.21.0", &base);
    assert!(result.is_ok());
    assert!(bin_dir.exists());
    assert!(bin_dir.is_dir());

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_version_symlink_atomicity() {
    use std::os::unix::fs::PermissionsExt;
    let base = make_temp_dir("atomicity");

    let tc_v1 = base.join("toolchains/node/1.0.0/bin");
    let tc_v2 = base.join("toolchains/node/2.0.0/bin");
    fs::create_dir_all(&tc_v1).unwrap();
    fs::create_dir_all(&tc_v2).unwrap();

    for name in &["node", "npm"] {
        let path1 = tc_v1.join(name);
        fs::write(&path1, "v1").unwrap();
        let mut perms = fs::metadata(&path1).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path1, perms).unwrap();

        let path2 = tc_v2.join(name);
        fs::write(&path2, "v2").unwrap();
        let mut perms = fs::metadata(&path2).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path2, perms).unwrap();
    }

    switch_version_in(&NodeTool, "1.0.0", &base).unwrap();
    let target1 = fs::read_link(base.join("current/node")).unwrap();

    switch_version_in(&NodeTool, "2.0.0", &base).unwrap();
    let target2 = fs::read_link(base.join("current/node")).unwrap();

    assert_ne!(target1, target2);
    assert!(target2.ends_with("toolchains/node/2.0.0"));

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_version_with_special_characters() {
    use std::os::unix::fs::PermissionsExt;
    let base = make_temp_dir("special_chars");

    let tc = base.join("toolchains/node/20.0.0-beta.1/bin");
    fs::create_dir_all(&tc).unwrap();
    for name in &["node", "npm"] {
        let path = tc.join(name);
        fs::write(&path, "beta").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }

    let result = switch_version_in(&NodeTool, "20.0.0-beta.1", &base);
    assert!(result.is_ok());

    let target = fs::read_link(base.join("current/node")).unwrap();
    assert!(target.to_string_lossy().contains("20.0.0-beta.1"));

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_multiple_tools_independently() {
    use std::os::unix::fs::PermissionsExt;
    let base = make_temp_dir("multi_tools");

    let node_tc = base.join("toolchains/node/20.0.0/bin");
    fs::create_dir_all(&node_tc).unwrap();
    let node_path = node_tc.join("node");
    fs::write(&node_path, "node").unwrap();
    let mut perms = fs::metadata(&node_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&node_path, perms).unwrap();

    let go_tc = base.join("toolchains/go/1.21.0/bin");
    fs::create_dir_all(&go_tc).unwrap();
    let go_path = go_tc.join("go");
    fs::write(&go_path, "go").unwrap();
    let mut perms = fs::metadata(&go_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&go_path, perms).unwrap();

    switch_version_in(&NodeTool, "20.0.0", &base).unwrap();
    switch_version_in(&GoTool, "1.21.0", &base).unwrap();

    assert!(base.join("current/node").exists());
    assert!(base.join("current/go").exists());
    assert!(base.join("bin/node").exists());
    assert!(base.join("bin/go").exists());

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_version_preserves_other_tools() {
    use std::os::unix::fs::PermissionsExt;
    let base = make_temp_dir("preserve_tools");

    let node_tc = base.join("toolchains/node/20.0.0/bin");
    fs::create_dir_all(&node_tc).unwrap();
    let node_path = node_tc.join("node");
    fs::write(&node_path, "node").unwrap();
    let mut perms = fs::metadata(&node_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&node_path, perms).unwrap();

    let go_tc = base.join("toolchains/go/1.21.0/bin");
    fs::create_dir_all(&go_tc).unwrap();
    let go_path = go_tc.join("go");
    fs::write(&go_path, "go").unwrap();
    let mut perms = fs::metadata(&go_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&go_path, perms).unwrap();

    switch_version_in(&NodeTool, "20.0.0", &base).unwrap();
    switch_version_in(&GoTool, "1.21.0", &base).unwrap();

    let node_tc2 = base.join("toolchains/node/21.0.0/bin");
    fs::create_dir_all(&node_tc2).unwrap();
    let node_path2 = node_tc2.join("node");
    fs::write(&node_path2, "node21").unwrap();
    let mut perms = fs::metadata(&node_path2).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&node_path2, perms).unwrap();

    switch_version_in(&NodeTool, "21.0.0", &base).unwrap();

    assert!(base.join("current/go").exists());
    assert!(base.join("bin/go").exists());

    let node_target = fs::read_link(base.join("current/node")).unwrap();
    assert!(node_target.to_string_lossy().contains("21.0.0"));

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_switch_rolls_back_when_bin_link_update_fails() {
    let base = make_temp_dir("rollback_on_bin_failure");

    let tc_v1 = base.join("toolchains/node/1.0.0/bin");
    let tc_v2 = base.join("toolchains/node/2.0.0/bin");
    fs::create_dir_all(&tc_v1).unwrap();
    fs::create_dir_all(&tc_v2).unwrap();

    for dir in [&tc_v1, &tc_v2] {
        for name in &["node", "npm", "npx"] {
            fs::write(dir.join(name), "fake").unwrap();
        }
    }

    switch_version_in(&NodeTool, "1.0.0", &base).unwrap();
    let _failure_guard = inject_test_failure(TestFailurePoint::BinLink("npm".to_string()));

    let err = switch_version_in(&NodeTool, "2.0.0", &base)
        .unwrap_err()
        .to_string();
    assert!(err.contains("Injected test failure"));

    let current_target = fs::read_link(base.join("current/node")).unwrap();
    assert!(current_target.ends_with("toolchains/node/1.0.0"));

    for name in &["node", "npm", "npx"] {
        let target = fs::read_link(base.join("bin").join(name)).unwrap();
        assert!(
            target.to_string_lossy().contains("/toolchains/node/1.0.0/"),
            "expected rollback for {}, got {}",
            name,
            target.display()
        );
    }

    let _ = fs::remove_dir_all(&base);
}
