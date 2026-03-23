use super::env::active_python_bin_in;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_find_active_python_bin_fallback() {
    let temp = TempDir::new().unwrap();
    let bin_dir = temp.path().join(".vex").join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    assert!(!bin_dir.join("python3").exists());
    assert_eq!(
        active_python_bin_in(temp.path().join(".vex").as_path()),
        PathBuf::from("python3")
    );
}

#[test]
fn test_find_active_python_bin_vex_bin() {
    let temp = TempDir::new().unwrap();
    let bin_dir = temp.path().join(".vex").join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let python_bin = bin_dir.join("python3");
    fs::write(&python_bin, "").unwrap();
    assert_eq!(
        active_python_bin_in(temp.path().join(".vex").as_path()),
        python_bin
    );
}
