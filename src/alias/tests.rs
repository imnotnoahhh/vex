use super::*;
use tempfile::TempDir;

#[test]
fn test_alias_resolution() {
    let temp = TempDir::new().unwrap();
    let manager = AliasManager::new(temp.path());

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();

    manager.set_global("node", "prod", "20.11.0").unwrap();
    assert_eq!(
        manager.resolve("node", "prod").unwrap(),
        Some("20.11.0".to_string())
    );

    manager.set_project("node", "prod", "21.0.0").unwrap();
    assert_eq!(
        manager.resolve("node", "prod").unwrap(),
        Some("21.0.0".to_string())
    );

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_alias_deletion() {
    let temp = TempDir::new().unwrap();
    let manager = AliasManager::new(temp.path());

    manager.set_global("node", "test", "20.0.0").unwrap();
    assert!(manager.delete_global("node", "test").unwrap());
    assert!(!manager.delete_global("node", "test").unwrap());
}
