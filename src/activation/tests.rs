use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_activation_plan_uses_project_venv_and_toolchain_bins() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let vex_dir = home.path().join(".vex");
    let toolchain_bin = vex_dir.join("toolchains/node/20.11.0/bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::create_dir_all(project.path().join(".venv/bin")).unwrap();
    fs::write(project.path().join(".tool-versions"), "node 20.11.0\n").unwrap();

    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", home.path());
    let plan = build_activation_plan(project.path()).unwrap();

    let path = exec_path(&plan);
    assert!(path.starts_with(project.path().join(".venv/bin").to_string_lossy().as_ref()));
    assert!(path.contains(toolchain_bin.to_string_lossy().as_ref()));
    assert!(path.contains(
        home.path()
            .join(".vex/npm/prefix/bin")
            .to_string_lossy()
            .as_ref()
    ));
    let expected_venv = project.path().join(".venv").display().to_string();
    assert_eq!(
        plan.set_env.get("VIRTUAL_ENV").cloned(),
        Some(expected_venv)
    );
    assert_eq!(
        plan.set_env.get("NPM_CONFIG_PREFIX").cloned(),
        Some(home.path().join(".vex/npm/prefix").display().to_string())
    );

    if let Some(value) = old_home {
        std::env::set_var("HOME", value);
    } else {
        std::env::remove_var("HOME");
    }
}

#[test]
fn test_activation_plan_uses_python_base_without_project_venv() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let vex_dir = home.path().join(".vex");
    let toolchain_bin = vex_dir.join("toolchains/python/3.13.3/bin");
    let base_bin = vex_dir.join("python/base/3.13.3/bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::create_dir_all(&base_bin).unwrap();
    fs::write(project.path().join(".tool-versions"), "python 3.13.3\n").unwrap();

    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", home.path());
    let plan = build_activation_plan(project.path()).unwrap();

    let shell = shell_path(&plan).unwrap();
    assert!(
        shell.starts_with(base_bin.to_string_lossy().as_ref()),
        "shell path was: {shell}"
    );
    let exec = exec_path(&plan);
    assert!(exec.contains(toolchain_bin.to_string_lossy().as_ref()));

    if let Some(value) = old_home {
        std::env::set_var("HOME", value);
    } else {
        std::env::remove_var("HOME");
    }
}

#[test]
fn test_activation_plan_hides_python_base_inside_project_venv() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let vex_dir = home.path().join(".vex");
    let toolchain_bin = vex_dir.join("toolchains/python/3.13.3/bin");
    let base_bin = vex_dir.join("python/base/3.13.3/bin");
    let venv_bin = project.path().join(".venv/bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::create_dir_all(&base_bin).unwrap();
    fs::create_dir_all(&venv_bin).unwrap();
    fs::write(project.path().join(".tool-versions"), "python 3.13.3\n").unwrap();

    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", home.path());
    let plan = build_activation_plan(project.path()).unwrap();

    let shell = shell_path(&plan).unwrap();
    assert!(
        shell.starts_with(venv_bin.to_string_lossy().as_ref()),
        "shell path was: {shell}"
    );
    assert!(!shell.contains(base_bin.to_string_lossy().as_ref()));
    let exec = exec_path(&plan);
    assert!(!exec.contains(base_bin.to_string_lossy().as_ref()));
    assert!(exec.contains(toolchain_bin.to_string_lossy().as_ref()));

    if let Some(value) = old_home {
        std::env::set_var("HOME", value);
    } else {
        std::env::remove_var("HOME");
    }
}
