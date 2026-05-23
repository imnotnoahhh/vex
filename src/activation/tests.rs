use super::*;
use std::fs;
use tempfile::TempDir;

fn with_home<T>(home: &std::path::Path, f: impl FnOnce() -> T) -> T {
    let _guard = crate::test_env::lock();
    let old_vex_home = std::env::var("VEX_HOME").ok();

    std::env::set_var("VEX_HOME", home.join(".vex"));
    let result = f();

    if let Some(value) = old_vex_home {
        std::env::set_var("VEX_HOME", value);
    } else {
        std::env::remove_var("VEX_HOME");
    }

    result
}

#[test]
fn test_activation_plan_uses_project_venv_and_toolchain_bins() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let vex_dir = home.path().join(".vex");
    let toolchain_bin = vex_dir.join("toolchains/node/20.11.0/bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::create_dir_all(project.path().join(".venv/bin")).unwrap();
    fs::write(project.path().join(".tool-versions"), "node 20.11.0\n").unwrap();

    with_home(home.path(), || {
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
        assert_eq!(
            plan.set_env.get("NPM_CONFIG_USERCONFIG").cloned(),
            Some(home.path().join(".vex/npm/npmrc").display().to_string())
        );
    });
}

#[test]
fn test_activation_plan_uses_python_base_without_project_venv() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let vex_dir = home.path().join(".vex");
    let toolchain_bin = vex_dir.join("toolchains/python/3.13.3/bin");
    let base_bin = vex_dir.join("python/base/3.13.3/bin");
    let user_bin = vex_dir.join("python/user/bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::create_dir_all(&base_bin).unwrap();
    fs::create_dir_all(&user_bin).unwrap();
    fs::write(project.path().join(".tool-versions"), "python 3.13.3\n").unwrap();

    with_home(home.path(), || {
        let plan = build_activation_plan(project.path()).unwrap();

        let shell = shell_path(&plan).unwrap();
        assert!(
            shell.starts_with(base_bin.to_string_lossy().as_ref()),
            "shell path was: {shell}"
        );
        assert!(shell.contains(user_bin.to_string_lossy().as_ref()));
        let exec = exec_path(&plan);
        assert!(exec.contains(toolchain_bin.to_string_lossy().as_ref()));
        assert_eq!(
            plan.set_env.get("PYTHONUSERBASE").cloned(),
            Some(vex_dir.join("python/user").display().to_string())
        );
        assert!(!plan.set_env.contains_key("PYTHONPATH"));
        assert!(plan.unset_env.contains(&"PYTHONPATH".to_string()));
    });
}

#[test]
fn test_activation_plan_unsets_hostile_java_env_vars() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let vex_dir = home.path().join(".vex");
    fs::create_dir_all(vex_dir.join("toolchains/java/25")).unwrap();
    fs::write(project.path().join(".tool-versions"), "java 25\n").unwrap();

    with_home(home.path(), || {
        let plan = build_activation_plan(project.path()).unwrap();

        assert!(!plan.set_env.contains_key("JAVA_TOOL_OPTIONS"));
        assert!(!plan.set_env.contains_key("_JAVA_OPTIONS"));
        assert!(plan.unset_env.contains(&"JAVA_TOOL_OPTIONS".to_string()));
        assert!(plan.unset_env.contains(&"_JAVA_OPTIONS".to_string()));
    });
}

#[test]
fn test_activation_plan_hides_python_base_inside_project_venv() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let vex_dir = home.path().join(".vex");
    let toolchain_bin = vex_dir.join("toolchains/python/3.13.3/bin");
    let base_bin = vex_dir.join("python/base/3.13.3/bin");
    let user_bin = vex_dir.join("python/user/bin");
    let venv_bin = project.path().join(".venv/bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::create_dir_all(&base_bin).unwrap();
    fs::create_dir_all(&user_bin).unwrap();
    fs::create_dir_all(&venv_bin).unwrap();
    fs::write(project.path().join(".tool-versions"), "python 3.13.3\n").unwrap();

    with_home(home.path(), || {
        let plan = build_activation_plan(project.path()).unwrap();

        let shell = shell_path(&plan).unwrap();
        assert!(
            shell.starts_with(venv_bin.to_string_lossy().as_ref()),
            "shell path was: {shell}"
        );
        assert!(!shell.contains(base_bin.to_string_lossy().as_ref()));
        assert!(!shell.contains(user_bin.to_string_lossy().as_ref()));
        let exec = exec_path(&plan);
        assert!(!exec.contains(base_bin.to_string_lossy().as_ref()));
        assert!(!exec.contains(user_bin.to_string_lossy().as_ref()));
        assert!(exec.contains(toolchain_bin.to_string_lossy().as_ref()));
    });
}

#[test]
fn test_activation_plan_prefers_project_node_modules_bin() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let nested = project.path().join("packages/app/src");
    let vex_dir = home.path().join(".vex");
    let toolchain_bin = vex_dir.join("toolchains/node/24.0.0/bin");
    let project_bin = project.path().join("node_modules/.bin");
    let npm_bin = vex_dir.join("npm/prefix/bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::create_dir_all(&project_bin).unwrap();
    fs::create_dir_all(&nested).unwrap();
    fs::write(project.path().join(".tool-versions"), "node 24.0.0\n").unwrap();

    with_home(home.path(), || {
        let plan = build_activation_plan(&nested).unwrap();

        let shell = shell_path(&plan).unwrap();
        assert!(
            shell.starts_with(project_bin.to_string_lossy().as_ref()),
            "shell path was: {shell}"
        );
        assert!(
            shell.find(project_bin.to_string_lossy().as_ref()).unwrap()
                < shell.find(npm_bin.to_string_lossy().as_ref()).unwrap(),
            "shell path was: {shell}"
        );

        let exec = exec_path(&plan);
        assert!(
            exec.starts_with(project_bin.to_string_lossy().as_ref()),
            "exec path was: {exec}"
        );
        assert!(exec.contains(toolchain_bin.to_string_lossy().as_ref()));
    });
}

#[test]
fn test_shell_activation_skips_unsafe_project_env_keys() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    fs::write(
        project.path().join(".vex.toml"),
        r#"
[env]
APP_ENV = "dev"
PYTHONPATH = "evil"
LD_PRELOAD = "evil"
"#,
    )
    .unwrap();

    with_home(home.path(), || {
        let plan = build_shell_activation_plan(project.path()).unwrap();

        assert_eq!(plan.set_env.get("APP_ENV"), Some(&"dev".to_string()));
        assert!(!plan.set_env.contains_key("PYTHONPATH"));
        assert!(!plan.set_env.contains_key("LD_PRELOAD"));
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.contains("skipped unsafe project env keys")));
    });
}

#[test]
fn test_shell_activation_unsets_project_env_after_leaving_project() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    fs::write(
        project.path().join(".vex.toml"),
        r#"
[env]
APP_ENV = "dev"
"#,
    )
    .unwrap();

    with_home(home.path(), || {
        let project_plan = build_shell_activation_plan(project.path()).unwrap();
        assert_eq!(
            project_plan.set_env.get("APP_ENV"),
            Some(&"dev".to_string())
        );

        let outside_plan = build_shell_activation_plan(outside.path()).unwrap();
        assert!(outside_plan.unset_env.contains(&"APP_ENV".to_string()));
        assert!(!outside_plan.set_env.contains_key("APP_ENV"));
    });
}
