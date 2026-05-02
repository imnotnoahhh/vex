use super::*;
use tempfile::TempDir;

#[test]
fn test_load_nearest_project_config() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("project");
    let nested = project.join("src/bin");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        project.join(".vex.toml"),
        r#"
[behavior]
auto_switch = false

[env]
RUST_LOG = "debug"

[commands]
test = "cargo test"
"#,
    )
    .unwrap();

    let loaded = load_nearest_project_config(&nested)
        .unwrap()
        .expect("project config should load");
    assert_eq!(loaded.root, project);
    assert!(!loaded.config.behavior.auto_switch.unwrap());
    assert_eq!(
        loaded.config.env.get("RUST_LOG").map(String::as_str),
        Some("debug")
    );
    assert_eq!(
        loaded.config.commands.get("test").map(String::as_str),
        Some("cargo test")
    );
}

#[test]
fn test_load_nearest_project_config_rejects_unsafe_env_key() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join(".vex.toml"),
        r#"
[env]
"EVIL; touch /tmp/vex-poc; #" = "x"
"#,
    )
    .unwrap();

    let error = load_nearest_project_config(&project).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Invalid environment variable name"),
        "unexpected error: {error}"
    );
}

#[test]
fn test_load_nearest_project_config_rejects_whitespace_env_key() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join(".vex.toml"),
        r#"
[env]
" FOO" = "x"
FOO = "y"
"#,
    )
    .unwrap();

    let error = load_nearest_project_config(&project).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Invalid environment variable name"),
        "unexpected error: {error}"
    );
}

#[test]
fn test_load_nearest_project_config_accepts_valid_env_keys() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join(".vex.toml"),
        r#"
[env]
RUST_LOG = "debug"
_VEX_TEST = "1"
VEX_TEST_2 = "ok"
"#,
    )
    .unwrap();

    let loaded = load_nearest_project_config(&project)
        .unwrap()
        .expect("project config should load");
    assert_eq!(loaded.config.env.len(), 3);
}

#[test]
fn test_find_nearest_venv() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("project");
    let nested = project.join("nested/deeper");
    fs::create_dir_all(project.join(".venv")).unwrap();
    fs::create_dir_all(&nested).unwrap();

    let venv = find_nearest_venv(&nested).expect("venv should be found");
    assert_eq!(venv, project.join(".venv"));
}

#[test]
fn test_find_nearest_node_modules_bin() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("project");
    let nested = project.join("packages/app/src");
    fs::create_dir_all(project.join("node_modules/.bin")).unwrap();
    fs::create_dir_all(&nested).unwrap();

    let bin = find_nearest_node_modules_bin(&nested).expect("node_modules/.bin should be found");
    assert_eq!(bin, project.join("node_modules/.bin"));
}
