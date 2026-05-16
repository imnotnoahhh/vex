use super::*;
use crate::tools::python::PYTHON_BUILD_STANDALONE_INTERNAL_ALIAS;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::TempDir;

fn write_executable(path: &Path) {
    fs::write(path, "#!/bin/sh\n").unwrap();
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

#[test]
fn collect_reports_node_shared_npm_globals() {
    let _guard = crate::test_env::lock();
    let home = TempDir::new().unwrap();
    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", home.path());

    let bin_dir = home.path().join(".vex/npm/prefix/bin");
    fs::create_dir_all(&bin_dir).unwrap();
    write_executable(&bin_dir.join("tsx"));

    let report = collect(Some("node")).unwrap();
    let tsx = report
        .entries
        .iter()
        .find(|entry| entry.tool == "node" && entry.name == "tsx")
        .expect("npm global CLI should be reported");
    assert_eq!(tsx.kind, "npm_global");
    assert_eq!(tsx.source, "shared npm globals");

    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    } else {
        std::env::remove_var("HOME");
    }
}

#[test]
fn collect_reports_go_and_rust_global_bins() {
    let _guard = crate::test_env::lock();
    let home = TempDir::new().unwrap();
    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", home.path());

    let go_bin = home.path().join(".vex/go/bin");
    let cargo_bin = home.path().join(".vex/cargo/bin");
    fs::create_dir_all(&go_bin).unwrap();
    fs::create_dir_all(&cargo_bin).unwrap();
    write_executable(&go_bin.join("gopls"));
    write_executable(&cargo_bin.join("cargo-audit"));

    let report = collect(None).unwrap();
    assert!(report
        .entries
        .iter()
        .any(|entry| entry.tool == "go" && entry.name == "gopls"));
    assert!(report
        .entries
        .iter()
        .any(|entry| entry.tool == "rust" && entry.name == "cargo-audit"));

    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    } else {
        std::env::remove_var("HOME");
    }
}

#[test]
fn collect_reports_java_build_state() {
    let _guard = crate::test_env::lock();
    let home = TempDir::new().unwrap();
    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", home.path());

    fs::create_dir_all(home.path().join(".m2/repository")).unwrap();
    fs::create_dir_all(home.path().join(".gradle/caches")).unwrap();

    let report = collect(Some("java")).unwrap();
    assert!(report
        .entries
        .iter()
        .any(|entry| entry.name == "maven-local-repository"));
    assert!(report
        .entries
        .iter()
        .any(|entry| entry.name == "gradle-caches"));

    let mvn_report = collect(Some("mvn")).unwrap();
    assert!(mvn_report
        .entries
        .iter()
        .any(|entry| entry.name == "maven-local-repository"));

    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    } else {
        std::env::remove_var("HOME");
    }
}

#[test]
fn collect_python_base_reports_user_clis_only() {
    let _guard = crate::test_env::lock();
    let home = TempDir::new().unwrap();
    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", home.path());

    let bin_dir = home.path().join(".vex/python/base/3.14.4/bin");
    let user_bin_dir = home.path().join(".vex/python/user/bin");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::create_dir_all(&user_bin_dir).unwrap();
    write_executable(&bin_dir.join("kaggle"));
    write_executable(&bin_dir.join("pip"));
    write_executable(&bin_dir.join("python3.14"));
    write_executable(&bin_dir.join(PYTHON_BUILD_STANDALONE_INTERNAL_ALIAS));
    write_executable(&user_bin_dir.join("black"));

    let report = collect(Some("python")).unwrap();
    assert!(report
        .entries
        .iter()
        .any(|entry| entry.tool == "python" && entry.name == "kaggle"));
    assert!(report.entries.iter().any(|entry| {
        entry.tool == "python"
            && entry.kind == "python_user_base"
            && entry.name == "black"
            && entry.source == "Python user base (pip --user)"
    }));
    assert!(!report.entries.iter().any(|entry| entry.name == "pip"
        || entry.name == "python3.14"
        || entry.name == "\u{1d70b}thon"));

    let pip_report = collect(Some("pip")).unwrap();
    assert!(pip_report
        .entries
        .iter()
        .any(|entry| entry.name == "kaggle"));
    assert!(pip_report.entries.iter().any(|entry| entry.name == "black"));

    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    } else {
        std::env::remove_var("HOME");
    }
}

#[test]
fn collect_accepts_official_package_manager_filters() {
    let _guard = crate::test_env::lock();
    let home = TempDir::new().unwrap();
    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", home.path());

    let npm_bin = home.path().join(".vex/npm/prefix/bin");
    let cargo_bin = home.path().join(".vex/cargo/bin");
    fs::create_dir_all(&npm_bin).unwrap();
    fs::create_dir_all(&cargo_bin).unwrap();
    write_executable(&npm_bin.join("eslint"));
    write_executable(&cargo_bin.join("cargo-audit"));

    let npm_report = collect(Some("npm")).unwrap();
    assert!(npm_report
        .entries
        .iter()
        .any(|entry| entry.tool == "node" && entry.name == "eslint"));

    let cargo_report = collect(Some("cargo")).unwrap();
    assert!(cargo_report
        .entries
        .iter()
        .any(|entry| entry.tool == "rust" && entry.name == "cargo-audit"));

    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    } else {
        std::env::remove_var("HOME");
    }
}
