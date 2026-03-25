use super::parse::{parse_team_config, validate_remote_team_config_response};
use super::source::{classify_source, SourceKind};
use super::*;
use std::collections::BTreeMap;
use std::fs;
use std::process::{Command, Stdio};
use tempfile::TempDir;

#[test]
fn test_parse_team_config_basic() {
    let config = r#"
version = 1

[tools]
node = "20.12.2"
python = "3.12.8"
"#;

    let versions = parse_team_config(config).unwrap();
    assert_eq!(
        versions,
        vec![
            ("node".to_string(), "20.12.2".to_string()),
            ("python".to_string(), "3.12.8".to_string())
        ]
    );
}

#[test]
fn test_parse_team_config_rejects_unexpected_fields() {
    let config = r#"
version = 1

[tools]
node = "20.12.2"

[commands]
build = "npm run build"
"#;

    let err = parse_team_config(config).unwrap_err().to_string();
    assert!(err.contains("unsupported top-level fields"));
}

#[test]
fn test_load_versions_from_local_team_config_prefers_local_tool_versions() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("vex-config.toml"),
        "version = 1\n\n[tools]\nnode = \"20.12.2\"\ngo = \"1.24.0\"\n",
    )
    .unwrap();
    fs::write(temp.path().join(".tool-versions"), "node 22.0\n").unwrap();

    let loaded = load_versions_from_source("vex-config.toml", temp.path(), false).unwrap();
    let versions: BTreeMap<_, _> = loaded.versions.into_iter().collect();
    assert_eq!(versions.get("node"), Some(&"22.0".to_string()));
    assert_eq!(versions.get("go"), Some(&"1.24.0".to_string()));
}

#[test]
fn test_load_versions_from_local_version_file() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("custom-versions.txt"), "node 20.11.0\n").unwrap();

    let loaded = load_versions_from_source("custom-versions.txt", temp.path(), false).unwrap();
    assert_eq!(
        loaded.versions,
        vec![("node".to_string(), "20.11.0".to_string())]
    );
}

#[test]
fn test_relative_source_is_resolved_from_current_dir_only() {
    let temp = TempDir::new().unwrap();
    let nested = temp.path().join("nested");
    fs::create_dir_all(&nested).unwrap();
    fs::write(temp.path().join("custom-versions.txt"), "go 1.24.0\n").unwrap();

    let err = load_versions_from_source("custom-versions.txt", &nested, false)
        .unwrap_err()
        .to_string();
    assert!(err.contains("Version file not found"));
    assert!(err.contains("nested/custom-versions.txt"));
}

#[test]
fn test_missing_team_config_file_reports_correct_error() {
    let temp = TempDir::new().unwrap();
    let err = load_versions_from_source("vex-config.toml", temp.path(), false)
        .unwrap_err()
        .to_string();
    assert!(err.contains("Team config file not found"));
}

#[test]
fn test_only_vex_config_toml_is_treated_as_team_config() {
    let temp = TempDir::new().unwrap();
    let cargo_toml = temp.path().join("Cargo.toml");
    fs::write(&cargo_toml, "[package]\nname = \"demo\"\n").unwrap();

    let source = classify_source("Cargo.toml", temp.path()).unwrap();
    assert!(matches!(source, SourceKind::VersionFile(path) if path == cargo_toml));
}

#[test]
fn test_load_versions_from_local_git_repo() {
    let temp = TempDir::new().unwrap();
    let repo = temp.path().join("team-config-repo");
    fs::create_dir_all(&repo).unwrap();

    Command::new("git")
        .args(["init", "--quiet"])
        .arg(&repo)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&repo)
        .args(["config", "user.email", "codex@example.com"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&repo)
        .args(["config", "user.name", "Codex"])
        .status()
        .unwrap();
    fs::write(
        repo.join("vex-config.toml"),
        "version = 1\n\n[tools]\nrust = \"stable\"\n",
    )
    .unwrap();
    Command::new("git")
        .current_dir(&repo)
        .args(["add", "vex-config.toml"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&repo)
        .args(["commit", "-m", "Add team config", "--quiet"])
        .status()
        .unwrap();

    let loaded = load_versions_from_source(repo.to_str().unwrap(), temp.path(), false).unwrap();
    assert_eq!(
        loaded.versions,
        vec![("rust".to_string(), "stable".to_string())]
    );
}

#[test]
fn test_validate_remote_team_config_rejects_html_content_type() {
    let err = validate_remote_team_config_response(
        "https://example.com/vex-config.toml",
        Some("text/html; charset=utf-8"),
        "<html><body>login</body></html>",
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("returned HTML content"));
}

#[test]
fn test_validate_remote_team_config_rejects_unexpected_content_type() {
    let err = validate_remote_team_config_response(
        "https://example.com/vex-config.toml",
        Some("application/json"),
        "{\"tools\":{}}",
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("unsupported content type"));
}
