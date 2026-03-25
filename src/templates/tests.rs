use super::write::rollback::{rollback_applied_writes, AppliedWrite};
use super::*;
use std::fs;
use tempfile::tempdir_in;
use tempfile::TempDir;

#[test]
fn test_list_templates_contains_all_core_templates() {
    let ids: Vec<_> = list_templates()
        .iter()
        .map(|template| template.id)
        .collect();
    assert!(ids.contains(&"node-typescript"));
    assert!(ids.contains(&"go-service"));
    assert!(ids.contains(&"java-basic"));
    assert!(ids.contains(&"rust-cli"));
    assert!(ids.contains(&"python-venv"));
}

#[test]
fn test_strict_mode_rejects_existing_files() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join(".tool-versions"), "node 20\n").unwrap();

    let result = init_template(temp.path(), "node-typescript", false, ConflictMode::Strict);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains(".tool-versions"));
    assert!(!temp.path().join("package.json").exists());
}

#[test]
fn test_add_only_merges_tool_versions_and_gitignore() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join(".tool-versions"), "rust stable\n").unwrap();
    fs::write(temp.path().join(".gitignore"), "target/\n").unwrap();

    init_template(temp.path(), "python-venv", false, ConflictMode::AddOnly).unwrap();

    let tool_versions = fs::read_to_string(temp.path().join(".tool-versions")).unwrap();
    assert!(tool_versions.contains("rust stable"));
    assert!(tool_versions.contains("python 3.12"));

    let gitignore = fs::read_to_string(temp.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains("target/"));
    assert!(gitignore.contains(".venv/"));
    assert!(temp.path().join(".vex.toml").exists());
    assert!(temp.path().join("src/main.py").exists());
}

#[test]
fn test_add_only_rejects_existing_vex_toml() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join(".tool-versions"), "").unwrap();
    fs::write(temp.path().join(".gitignore"), "").unwrap();
    fs::write(
        temp.path().join(".vex.toml"),
        "[commands]\nrun = \"true\"\n",
    )
    .unwrap();

    let result = init_template(temp.path(), "rust-cli", false, ConflictMode::AddOnly);
    assert!(result.is_err());
    assert!(!temp.path().join("Cargo.toml").exists());
}

#[test]
fn test_dry_run_does_not_write_files() {
    let temp = TempDir::new().unwrap();
    init_template(temp.path(), "go-service", true, ConflictMode::Strict).unwrap();
    assert!(!temp.path().join("go.mod").exists());
    assert!(!temp.path().join(".tool-versions").exists());
}

#[test]
fn test_strict_mode_rolls_back_files_when_write_fails() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("src"), "blocking file").unwrap();

    let result = init_template(temp.path(), "rust-cli", false, ConflictMode::Strict);
    assert!(result.is_err());
    assert!(!temp.path().join(".tool-versions").exists());
    assert!(!temp.path().join(".vex.toml").exists());
    assert!(!temp.path().join("Cargo.toml").exists());
    assert_eq!(
        fs::read_to_string(temp.path().join("src")).unwrap(),
        "blocking file"
    );
}

#[test]
fn test_add_only_rolls_back_merged_files_when_write_fails() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join(".tool-versions"), "rust stable\n").unwrap();
    fs::write(temp.path().join(".gitignore"), "target/\n").unwrap();
    fs::write(temp.path().join("src"), "blocking file").unwrap();

    let result = init_template(temp.path(), "python-venv", false, ConflictMode::AddOnly);
    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(temp.path().join(".tool-versions")).unwrap(),
        "rust stable\n"
    );
    assert_eq!(
        fs::read_to_string(temp.path().join(".gitignore")).unwrap(),
        "target/\n"
    );
    assert!(!temp.path().join(".vex.toml").exists());
    assert!(!temp.path().join("requirements.lock").exists());
    assert_eq!(
        fs::read_to_string(temp.path().join("src")).unwrap(),
        "blocking file"
    );
}

#[test]
fn test_rollback_removes_created_empty_directories() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("tests"), "blocking file").unwrap();

    let result = init_template(temp.path(), "rust-cli", false, ConflictMode::Strict);
    assert!(result.is_err());
    assert!(!temp.path().join("src").exists());
    assert_eq!(
        fs::read_to_string(temp.path().join("tests")).unwrap(),
        "blocking file"
    );
}

#[test]
fn test_rollback_best_effort_continues_after_error() {
    let temp = TempDir::new().unwrap();
    let first = temp.path().join("first.txt");
    let second = temp.path().join("second.txt");
    fs::write(&first, "new").unwrap();
    fs::write(&second, "new").unwrap();
    fs::remove_file(&first).unwrap();
    fs::create_dir(&first).unwrap();

    let staging_dir = tempdir_in(temp.path()).unwrap();
    let applied = vec![
        AppliedWrite {
            path: second.clone(),
            original_contents: None,
        },
        AppliedWrite {
            path: first.clone(),
            original_contents: None,
        },
    ];

    let err = rollback_applied_writes(staging_dir.path(), &applied, &[]).unwrap_err();
    assert!(err.to_string().contains("Rollback incomplete"));
    assert!(!second.exists(), "second file should still be removed");
    assert!(first.is_dir(), "failing path should remain unchanged");
}
