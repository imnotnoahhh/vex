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
fn test_find_nearest_venv() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("project");
    let nested = project.join("nested/deeper");
    fs::create_dir_all(project.join(".venv")).unwrap();
    fs::create_dir_all(&nested).unwrap();

    let venv = find_nearest_venv(&nested).expect("venv should be found");
    assert_eq!(venv, project.join(".venv"));
}
