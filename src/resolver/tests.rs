use super::*;
use std::fs;

#[test]
fn test_parse_tool_versions_basic() {
    let content = "node 20.11.0\ngo 1.23.5\njava 21\nrust 1.93.1\n";
    let result = parse_tool_versions(content);
    assert_eq!(
        result,
        vec![
            ("node".into(), "20.11.0".into()),
            ("go".into(), "1.23.5".into()),
            ("java".into(), "21".into()),
            ("rust".into(), "1.93.1".into()),
        ]
    );
}

#[test]
fn test_parse_tool_versions_with_comments() {
    let content = "# project versions\nnode 20.11.0\n\n# Go version\ngo 1.23.5\n";
    let result = parse_tool_versions(content);
    assert_eq!(
        result,
        vec![
            ("node".into(), "20.11.0".into()),
            ("go".into(), "1.23.5".into()),
        ]
    );
}

#[test]
fn test_parse_tool_versions_empty() {
    let result = parse_tool_versions("");
    assert!(result.is_empty());
}

#[test]
fn test_parse_tool_versions_extra_whitespace() {
    let content = "  node   20.11.0  \n  go   1.23.5  ";
    let result = parse_tool_versions(content);
    assert_eq!(
        result,
        vec![
            ("node".into(), "20.11.0".into()),
            ("go".into(), "1.23.5".into()),
        ]
    );
}

#[test]
fn test_parse_tool_versions_only_tool_no_version() {
    let content = "node\ngo 1.23.5";
    let result = parse_tool_versions(content);
    assert_eq!(result, vec![("go".into(), "1.23.5".into())]);
}

#[test]
fn test_resolve_version_from_file() {
    let dir = std::env::temp_dir().join("vex_test_resolve");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(".node-version"), "20.11.0\n").unwrap();

    let result = resolve_version("node", &dir);
    assert_eq!(result, Some("20.11.0".into()));

    let result = resolve_version("go", &dir);
    assert_eq!(result, None);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_version_tool_versions_priority() {
    let dir = std::env::temp_dir().join("vex_test_resolve_priority");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join(".tool-versions"), "node 22.0.0\n").unwrap();
    fs::write(dir.join(".node-version"), "20.11.0\n").unwrap();

    let result = resolve_version("node", &dir);
    assert_eq!(result, Some("22.0.0".into()));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_versions_all() {
    let dir = std::env::temp_dir().join("vex_test_resolve_all");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join(".tool-versions"), "node 20.11.0\ngo 1.23.5\n").unwrap();
    fs::write(dir.join(".java-version"), "21\n").unwrap();

    let versions = resolve_versions(&dir);
    assert_eq!(versions.get("node"), Some(&"20.11.0".into()));
    assert_eq!(versions.get("go"), Some(&"1.23.5".into()));
    assert_eq!(versions.get("java"), Some(&"21".into()));
    assert_eq!(versions.get("rust"), None);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_version_parent_dir() {
    let parent = std::env::temp_dir().join("vex_test_parent");
    let child = parent.join("subdir");
    let _ = fs::remove_dir_all(&parent);
    fs::create_dir_all(&child).unwrap();

    fs::write(parent.join(".node-version"), "20.11.0\n").unwrap();

    let result = resolve_version("node", &child);
    assert_eq!(result, Some("20.11.0".into()));

    let _ = fs::remove_dir_all(&parent);
}

#[test]
fn test_parse_tool_versions_malformed() {
    let content = "   \n\t\n";
    let result = parse_tool_versions(content);
    assert!(result.is_empty());

    let content = "node 20.11.0\ninvalid\ngo 1.23.5\n";
    let result = parse_tool_versions(content);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_resolve_version_nvmrc() {
    let dir = std::env::temp_dir().join("vex_test_nvmrc");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join(".nvmrc"), "18.0.0\n").unwrap();

    let result = resolve_version("node", &dir);
    assert_eq!(result, Some("18.0.0".into()));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_version_go_version() {
    let dir = std::env::temp_dir().join("vex_test_go_version");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join(".go-version"), "1.21.0\n").unwrap();

    let result = resolve_version("go", &dir);
    assert_eq!(result, Some("1.21.0".into()));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_version_java_version() {
    let dir = std::env::temp_dir().join("vex_test_java_version");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join(".java-version"), "17\n").unwrap();

    let result = resolve_version("java", &dir);
    assert_eq!(result, Some("17".into()));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_version_rust_toolchain() {
    let dir = std::env::temp_dir().join("vex_test_rust_toolchain");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join(".rust-toolchain"), "1.70.0\n").unwrap();

    let result = resolve_version("rust", &dir);
    assert_eq!(result, Some("1.70.0".into()));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_version_python_version() {
    let dir = std::env::temp_dir().join("vex_test_python_version");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join(".python-version"), "3.11.0\n").unwrap();

    let result = resolve_version("python", &dir);
    assert_eq!(result, Some("3.11.0".into()));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_version_with_whitespace() {
    let dir = std::env::temp_dir().join("vex_test_whitespace");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join(".node-version"), "  20.11.0  \n").unwrap();

    let result = resolve_version("node", &dir);
    assert_eq!(result, Some("20.11.0".into()));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_versions_nested_directories() {
    let root = std::env::temp_dir().join("vex_test_nested");
    let level1 = root.join("level1");
    let level2 = level1.join("level2");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&level2).unwrap();

    fs::write(root.join(".tool-versions"), "node 20.0.0\n").unwrap();
    fs::write(level1.join(".tool-versions"), "go 1.21.0\n").unwrap();

    let versions = resolve_versions(&level2);
    assert_eq!(versions.get("go"), Some(&"1.21.0".into()));
    let node_result = resolve_version("node", &level2);
    assert_eq!(node_result, Some("20.0.0".into()));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_resolve_local_tool_versions_only_ignores_language_specific_files() {
    let root = std::env::temp_dir().join("vex_test_local_tool_versions_only");
    let nested = root.join("nested");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&nested).unwrap();

    fs::write(root.join(".tool-versions"), "node 20.0.0\ngo 1.24.0\n").unwrap();
    fs::write(nested.join(".node-version"), "22.0.0\n").unwrap();
    fs::write(nested.join(".python-version"), "3.12.8\n").unwrap();

    let versions = resolve_local_tool_versions_only(&nested);
    assert_eq!(versions.get("node"), Some(&"20.0.0".into()));
    assert_eq!(versions.get("go"), Some(&"1.24.0".into()));
    assert!(!versions.contains_key("python"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_parse_tool_versions_multiple_spaces() {
    let content = "node    20.11.0\ngo\t\t1.23.5\n";
    let result = parse_tool_versions(content);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], ("node".into(), "20.11.0".into()));
    assert_eq!(result[1], ("go".into(), "1.23.5".into()));
}
