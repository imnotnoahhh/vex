use super::uninstall::uninstall;
use std::fs;

#[test]
fn test_uninstall_version_not_found() {
    let temp = tempfile::TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join(".vex").join("toolchains")).unwrap();
    std::env::set_var("HOME", temp.path());
    let result = uninstall("node", "99.0.0");
    assert!(result.is_err());
}
