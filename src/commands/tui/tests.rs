#[test]
fn test_read_tool_versions_empty() {
    use std::path::PathBuf;
    let path = PathBuf::from("/nonexistent/path");
    let versions = crate::resolver::read_tool_versions_file(&path);
    assert!(versions.is_empty());
}

#[test]
fn test_read_tool_versions_with_content() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "node 20.11.0").unwrap();
    writeln!(file, "go 1.23.5").unwrap();
    writeln!(file, "# comment").unwrap();
    writeln!(file).unwrap();
    file.flush().unwrap();

    let versions = crate::resolver::read_tool_versions_file(file.path());
    assert_eq!(versions.len(), 2);
    assert_eq!(versions.get("node"), Some(&"20.11.0".to_string()));
    assert_eq!(versions.get("go"), Some(&"1.23.5".to_string()));
}

#[test]
fn test_path_size_empty_dir() {
    use tempfile::TempDir;
    let temp_dir = TempDir::new().unwrap();
    let size = crate::fs_utils::path_size(temp_dir.path());
    assert_eq!(size, 0);
}

#[test]
fn test_path_size_with_file() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "hello world").unwrap();

    let size = crate::fs_utils::path_size(temp_dir.path());
    assert!(size > 0);
}
