use super::*;

#[test]
fn test_list_installed_no_toolchains_dir() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    std::env::set_var("HOME", temp.path());
    let result = commands::versions::list_installed("node", output::OutputMode::Text);
    assert!(result.is_ok());
}

#[test]
fn test_list_installed_empty_dir() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".vex").join("toolchains").join("node")).unwrap();
    std::env::set_var("HOME", temp.path());
    let result = commands::versions::list_installed("node", output::OutputMode::Text);
    assert!(result.is_ok());
}

#[test]
fn test_show_current_no_current_dir() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".vex")).unwrap();
    std::env::set_var("HOME", temp.path());
    let result = commands::current::show(output::OutputMode::Text);
    assert!(result.is_ok());
}

#[test]
fn test_show_current_empty_current_dir() {
    use tempfile::TempDir;

    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".vex").join("current")).unwrap();
    std::env::set_var("HOME", temp.path());
    let result = commands::current::show(output::OutputMode::Text);
    assert!(result.is_ok());
}
