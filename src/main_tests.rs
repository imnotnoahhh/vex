use super::*;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_home<T>(home: &std::path::Path, f: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK.lock().unwrap();
    let original_home = std::env::var("HOME").ok();

    std::env::set_var("HOME", home);
    let result = f();

    if let Some(value) = original_home {
        std::env::set_var("HOME", value);
    } else {
        std::env::remove_var("HOME");
    }

    result
}

#[test]
fn test_list_installed_no_toolchains_dir() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    let result =
        with_home(temp.path(), || commands::versions::list_installed("node", output::OutputMode::Text));
    assert!(result.is_ok());
}

#[test]
fn test_list_installed_empty_dir() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".vex").join("toolchains").join("node")).unwrap();
    let result =
        with_home(temp.path(), || commands::versions::list_installed("node", output::OutputMode::Text));
    assert!(result.is_ok());
}

#[test]
fn test_show_current_no_current_dir() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".vex")).unwrap();
    let result = with_home(temp.path(), || commands::current::show(output::OutputMode::Text));
    assert!(result.is_ok());
}

#[test]
fn test_show_current_empty_current_dir() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".vex").join("current")).unwrap();
    let result = with_home(temp.path(), || commands::current::show(output::OutputMode::Text));
    assert!(result.is_ok());
}
