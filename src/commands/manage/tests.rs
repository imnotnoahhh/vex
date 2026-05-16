use super::uninstall::uninstall;
use std::fs;

#[test]
fn test_uninstall_version_not_found() {
    let _guard = crate::test_env::lock();
    let temp = tempfile::TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join(".vex").join("toolchains")).unwrap();
    let old_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", temp.path());
    let result = uninstall("node", "99.0.0");
    if let Some(value) = old_home {
        std::env::set_var("HOME", value);
    } else {
        std::env::remove_var("HOME");
    }
    assert!(result.is_err());
}
