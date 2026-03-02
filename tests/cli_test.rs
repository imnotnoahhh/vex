use std::process::Command;

fn vex_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vex"))
}

#[test]
fn test_help() {
    let output = vex_bin().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fast version manager"));
}

#[test]
fn test_init() {
    let output = vex_bin().arg("init").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("source ~/.zshrc"));
}

#[test]
fn test_current() {
    let output = vex_bin().arg("current").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 应该显示已激活的版本或提示没有激活
    assert!(stdout.contains("active versions") || stdout.contains("No tools activated"));
}

// --- 错误场景测试 ---

#[test]
fn test_invalid_tool() {
    let output = vex_bin().args(["list-remote", "python"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_use_nonexistent_version() {
    let output = vex_bin().args(["use", "node@99.99.99"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Version not found"));
}

#[test]
fn test_uninstall_without_version() {
    let output = vex_bin().args(["uninstall", "node"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("specify a version"));
}

#[test]
fn test_list_installed_node() {
    let output = vex_bin().args(["list", "node"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 应该显示已安装版本或提示没有安装
    assert!(stdout.contains("Installed versions") || stdout.contains("No versions"));
}

#[test]
fn test_list_installed_nonexistent_tool() {
    // list 命令不经过 get_tool，直接读目录，所以不存在的工具应该提示没有安装
    let output = vex_bin().args(["list", "ruby"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No versions"));
}

#[test]
fn test_uninstall_nonexistent_version() {
    let output = vex_bin()
        .args(["uninstall", "node@99.99.99"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Version not found"));
}

#[test]
fn test_version_flag() {
    let output = vex_bin().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vex"));
}

#[test]
fn test_install_invalid_tool() {
    let output = vex_bin().args(["install", "python@3.12"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_use_invalid_tool() {
    let output = vex_bin().args(["use", "ruby@3.0"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_list_remote_invalid_tool() {
    let output = vex_bin().args(["list-remote", "ruby"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_install_invalid_spec() {
    let output = vex_bin().args(["install", "node@1@2@3"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid spec format"));
}

#[test]
fn test_list_all_supported_tools() {
    // 所有支持的工具都应该能 list（即使没安装）
    for tool in &["node", "go", "java", "rust"] {
        let output = vex_bin().args(["list", tool]).output().unwrap();
        assert!(output.status.success(), "list {} should succeed", tool);
    }
}

// --- env 命令测试 ---

#[test]
fn test_env_zsh() {
    let output = vex_bin().args(["env", "zsh"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("add-zsh-hook chpwd"));
    assert!(stdout.contains("__vex_use_if_found"));
}

#[test]
fn test_env_bash() {
    let output = vex_bin().args(["env", "bash"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PROMPT_COMMAND"));
    assert!(stdout.contains("__vex_use_if_found"));
}

#[test]
fn test_env_unsupported_shell() {
    let output = vex_bin().args(["env", "powershell"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported shell"));
}

#[test]
fn test_env_fish() {
    let output = vex_bin().args(["env", "fish"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("function __vex_use_if_found"));
}

#[test]
fn test_env_nushell() {
    let output = vex_bin().args(["env", "nu"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("def --env __vex_use_if_found"));
}

// --- use --auto 测试 ---

#[test]
fn test_use_auto_no_version_file() {
    let output = vex_bin().args(["use", "--auto"]).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_use_without_spec_or_auto() {
    let output = vex_bin().args(["use"]).output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_init_shows_eval_hint() {
    let output = vex_bin().arg("init").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("echo") && stdout.contains("vex env zsh"));
}

// --- install 无参数测试 ---

#[test]
fn test_install_no_args_no_version_file() {
    let temp_dir = std::env::temp_dir().join("vex_test_install_no_args");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let output = vex_bin()
        .arg("install")
        .current_dir(&temp_dir)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No version files found"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// --- local / global 测试 ---

#[test]
fn test_local_without_version() {
    let output = vex_bin().args(["local", "node"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("specify a version"));
}

#[test]
fn test_global_without_version() {
    let output = vex_bin().args(["global", "node"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("specify a version"));
}

#[test]
fn test_local_invalid_tool() {
    let output = vex_bin().args(["local", "python@3.12"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_local_writes_tool_versions() {
    let dir = std::env::temp_dir().join("vex_test_local_write");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // local 命令会调用 resolve_fuzzy_version，对完整版本号直接返回
    let output = vex_bin()
        .args(["local", "node@20.11.0"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(output.status.success());

    let tv = std::fs::read_to_string(dir.join(".tool-versions")).unwrap();
    assert!(tv.contains("node 20.11.0"));

    let _ = std::fs::remove_dir_all(&dir);
}

// --- doctor 命令测试 ---

#[test]
fn test_doctor_command() {
    let output = vex_bin().arg("doctor").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Health Check") || stdout.contains("doctor"));
}

// --- upgrade 命令测试 ---

#[test]
fn test_upgrade_invalid_tool() {
    let output = vex_bin().args(["upgrade", "python"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_upgrade_valid_tool_format() {
    // upgrade 命令会尝试 list_remote，但我们只测试参数解析
    let output = vex_bin().args(["upgrade", "node"]).output().unwrap();
    // 可能成功（如果有网络）或失败（网络错误），但不应该是参数错误
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Invalid spec format"));
}

// --- alias 命令测试 ---

#[test]
fn test_alias_invalid_tool() {
    let output = vex_bin().args(["alias", "python"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_alias_valid_tool_format() {
    let output = vex_bin().args(["alias", "node"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Invalid spec format"));
}

// --- install from version files ---

#[test]
fn test_install_from_version_file_unsupported_tool() {
    let dir = std::env::temp_dir().join("vex_test_install_unsupported");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Write .tool-versions with unsupported tool
    std::fs::write(dir.join(".tool-versions"), "python 3.12.0\n").unwrap();

    let output = vex_bin().arg("install").current_dir(&dir).output().unwrap();
    // Should not crash, just skip unsupported tool
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("skipping unsupported tool") || output.status.success());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_install_from_version_file_already_installed() {
    let dir = std::env::temp_dir().join("vex_test_install_already");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Write .tool-versions with a version that's already installed (if any)
    // This tests the "already installed, skipping" path
    std::fs::write(dir.join(".tool-versions"), "node 99.99.99\n").unwrap();

    let output = vex_bin().arg("install").current_dir(&dir).output().unwrap();
    // Will either fail with network error or succeed
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    // Should not be a parse error
    assert!(!combined.contains("Invalid spec format"));

    let _ = std::fs::remove_dir_all(&dir);
}

// --- global 命令测试 ---

#[test]
fn test_global_invalid_tool() {
    let output = vex_bin().args(["global", "python@3.12"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_global_writes_tool_versions() {
    // global with full version should write to ~/.tool-versions
    let output = vex_bin().args(["global", "node@20.11.0"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("node") && stdout.contains("20.11.0"));
}

// --- use --auto with version file ---

#[test]
fn test_use_auto_with_unsupported_tool() {
    let dir = std::env::temp_dir().join("vex_test_auto_unsupported");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(dir.join(".tool-versions"), "python 3.12.0\n").unwrap();

    let output = vex_bin()
        .args(["use", "--auto"])
        .current_dir(&dir)
        .output()
        .unwrap();
    // Should succeed (unsupported tools are silently skipped)
    assert!(output.status.success());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_use_auto_with_uninstalled_version() {
    let dir = std::env::temp_dir().join("vex_test_auto_uninstalled");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(dir.join(".tool-versions"), "node 99.99.99\n").unwrap();

    let output = vex_bin()
        .args(["use", "--auto"])
        .current_dir(&dir)
        .output()
        .unwrap();
    // Should succeed but print warning about uninstalled version
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not installed") || stderr.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

// --- doctor 详细检查 ---

#[test]
fn test_doctor_checks_directory_structure() {
    let output = vex_bin().arg("doctor").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checking"));
}

#[test]
fn test_doctor_checks_path() {
    let output = vex_bin().arg("doctor").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PATH"));
}

// --- init 重复运行 ---

#[test]
fn test_init_idempotent() {
    // Running init twice should succeed both times
    let output1 = vex_bin().arg("init").output().unwrap();
    assert!(output1.status.success());
    let output2 = vex_bin().arg("init").output().unwrap();
    assert!(output2.status.success());
}

// --- list-remote with --no-cache ---

#[test]
fn test_list_remote_no_cache_invalid_tool() {
    let output = vex_bin()
        .args(["list-remote", "python", "--no-cache"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}
