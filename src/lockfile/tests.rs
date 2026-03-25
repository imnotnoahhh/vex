use super::*;
use tempfile::TempDir;

#[test]
fn test_lockfile_new() {
    let lockfile = Lockfile::new();
    assert_eq!(lockfile.version, 1);
    assert!(lockfile.tools.is_empty());
}

#[test]
fn test_lockfile_add_tool() {
    let mut lockfile = Lockfile::new();
    lockfile.add_tool(
        "node".to_string(),
        LockEntry {
            version: "20.11.0".to_string(),
            sha256: Some("abc123".to_string()),
            url: None,
        },
    );
    assert_eq!(lockfile.tools.len(), 1);
    assert_eq!(lockfile.get_tool("node").unwrap().version, "20.11.0");
}

#[test]
fn test_lockfile_serialization() {
    let mut lockfile = Lockfile::new();
    lockfile.add_tool(
        "node".to_string(),
        LockEntry {
            version: "20.11.0".to_string(),
            sha256: Some("abc123".to_string()),
            url: Some("https://example.com/node".to_string()),
        },
    );

    let serialized = lockfile.to_string().unwrap();
    let deserialized = Lockfile::from_str(&serialized).unwrap();

    assert_eq!(deserialized.version, 1);
    assert_eq!(deserialized.tools.len(), 1);
    let entry = deserialized.get_tool("node").unwrap();
    assert_eq!(entry.version, "20.11.0");
    assert_eq!(entry.sha256.as_deref(), Some("abc123"));
}

#[test]
fn test_lockfile_save_and_load() {
    let temp = TempDir::new().unwrap();
    let mut lockfile = Lockfile::new();
    lockfile.add_tool(
        "go".to_string(),
        LockEntry {
            version: "1.23.5".to_string(),
            sha256: None,
            url: None,
        },
    );

    let path = lockfile.save_to_dir(temp.path()).unwrap();
    assert!(path.exists());

    let loaded = Lockfile::load_from_dir(temp.path()).unwrap().unwrap();
    assert_eq!(loaded.tools.len(), 1);
    assert_eq!(loaded.get_tool("go").unwrap().version, "1.23.5");
}

#[test]
fn test_lockfile_find_in_ancestors() {
    let temp = TempDir::new().unwrap();
    let parent = temp.path();
    let child = parent.join("subdir");
    fs::create_dir_all(&child).unwrap();

    let mut lockfile = Lockfile::new();
    lockfile.add_tool(
        "rust".to_string(),
        LockEntry {
            version: "1.93.1".to_string(),
            sha256: None,
            url: None,
        },
    );
    lockfile.save_to_dir(parent).unwrap();

    let found = Lockfile::find_in_ancestors(&child);
    assert!(found.is_some());
    assert_eq!(found.unwrap(), parent.join(LOCKFILE_NAME));
}

#[test]
fn test_lockfile_optional_fields() {
    let entry = LockEntry {
        version: "1.0.0".to_string(),
        sha256: None,
        url: None,
    };

    let mut lockfile = Lockfile::new();
    lockfile.add_tool("test".to_string(), entry);

    let serialized = lockfile.to_string().unwrap();
    assert!(!serialized.contains("sha256"));
    assert!(!serialized.contains("url"));
}
