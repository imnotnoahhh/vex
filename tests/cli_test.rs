use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, os::unix::fs::PermissionsExt};

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn vex_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vex"))
}

fn fresh_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_executable_script(path: &std::path::Path, body: &str) {
    fs::write(path, body).unwrap();
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

fn seed_remote_cache(home: &std::path::Path, tool: &str, versions: &[&str]) {
    let cache_dir = home.join(".vex/cache");
    fs::create_dir_all(&cache_dir).unwrap();
    let cached_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let versions_json = versions
        .iter()
        .map(|version| format!(r#"{{"version":"{}","lts":null}}"#, version))
        .collect::<Vec<_>>()
        .join(",");
    let json = format!(
        r#"{{"versions":[{}],"cached_at":{}}}"#,
        versions_json, cached_at
    );
    fs::write(cache_dir.join(format!("remote-{}.json", tool)), json).unwrap();
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
    let home = fresh_temp_dir("vex_test_init");
    let output = vex_bin().arg("init").env("HOME", &home).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vex init --shell auto"));
    assert!(home.join(".vex/npm/prefix/bin").exists());

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_init_list_templates() {
    let output = vex_bin()
        .args(["init", "--list-templates"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("node-typescript"));
    assert!(stdout.contains("python-venv"));
}

#[test]
fn test_init_list_templates_conflicts_with_dry_run() {
    let output = vex_bin()
        .args(["init", "--list-templates", "--dry-run"])
        .output()
        .unwrap();
    assert!(!output.status.success(), "{:?}", output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--dry-run"));
    assert!(stderr.contains("--list-templates"));
}

#[test]
fn test_init_template_dry_run_does_not_write_files() {
    let project = fresh_temp_dir("vex_test_template_dry_run");

    let output = vex_bin()
        .args(["init", "--template", "python-venv", "--dry-run"])
        .current_dir(&project)
        .output()
        .unwrap();

    assert!(output.status.success(), "{:?}", output);
    assert!(!project.join(".tool-versions").exists());
    assert!(!project.join(".vex.toml").exists());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No files were written"));

    let _ = std::fs::remove_dir_all(&project);
}

#[test]
fn test_init_template_add_only_merges_safe_files() {
    let project = fresh_temp_dir("vex_test_template_add_only");
    fs::write(project.join(".tool-versions"), "rust stable\n").unwrap();
    fs::write(project.join(".gitignore"), "target/\n").unwrap();

    let output = vex_bin()
        .args(["init", "--template", "python-venv", "--add-only"])
        .current_dir(&project)
        .output()
        .unwrap();

    assert!(output.status.success(), "{:?}", output);
    let tool_versions = fs::read_to_string(project.join(".tool-versions")).unwrap();
    assert!(tool_versions.contains("rust stable"));
    assert!(tool_versions.contains("python 3.12"));
    let gitignore = fs::read_to_string(project.join(".gitignore")).unwrap();
    assert!(gitignore.contains("target/"));
    assert!(gitignore.contains(".venv/"));
    assert!(project.join(".vex.toml").exists());
    assert!(project.join("src/main.py").exists());

    let _ = std::fs::remove_dir_all(&project);
}

#[test]
fn test_current() {
    let output = vex_bin().arg("current").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 应该显示已激活的版本或提示没有激活
    assert!(stdout.contains("active versions") || stdout.contains("No tools activated"));
}

#[test]
fn test_current_json() {
    let home = fresh_temp_dir("vex_test_current_json");
    let output = vex_bin()
        .args(["current", "--json"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.get("cwd").is_some());
    assert!(parsed.get("tools").unwrap().is_array());

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_env_exports_include_captured_user_state() {
    let home = fresh_temp_dir("vex_test_env_exports_home");
    let project = fresh_temp_dir("vex_test_env_exports_project");
    let toolchain_bin = home.join(".vex/toolchains/node/20.11.0/bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::create_dir_all(project.join(".venv/bin")).unwrap();
    fs::write(project.join(".tool-versions"), "node 20.11.0\n").unwrap();

    let output = vex_bin()
        .args(["env", "zsh", "--exports"])
        .env("HOME", &home)
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("export VIRTUAL_ENV="));
    assert!(stdout.contains(".vex/bin"));
    assert!(stdout.contains(".vex/npm/prefix/bin"));
    assert!(stdout.contains("NPM_CONFIG_PREFIX"));

    let _ = fs::remove_dir_all(&home);
    let _ = fs::remove_dir_all(&project);
}

#[test]
fn test_repair_migrate_home_dry_run_and_apply() {
    let home = fresh_temp_dir("vex_test_repair_home");
    fs::write(home.join(".tool-versions"), "node 20\n").unwrap();
    fs::create_dir_all(home.join(".cargo/bin")).unwrap();

    let dry_run = vex_bin()
        .args(["repair", "migrate-home"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(dry_run.status.success(), "{:?}", dry_run);
    let dry_stdout = String::from_utf8_lossy(&dry_run.stdout);
    assert!(dry_stdout.contains("Dry run complete"));
    assert!(dry_stdout.contains(".tool-versions"));

    let apply = vex_bin()
        .args(["repair", "migrate-home", "--apply"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(apply.status.success(), "{:?}", apply);
    assert!(home.join(".vex/tool-versions").exists());
    assert!(home.join(".vex/cargo").exists());
    assert!(!home.join(".tool-versions").exists());

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn test_doctor_reports_home_hygiene_and_repair_hint() {
    let home = fresh_temp_dir("vex_test_doctor_home");
    fs::create_dir_all(home.join(".vex/bin")).unwrap();
    fs::create_dir_all(home.join(".cargo")).unwrap();

    let output = vex_bin()
        .arg("doctor")
        .env("HOME", &home)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", home.join(".vex/bin").display()),
        )
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("home hygiene"));
    assert!(stdout.contains("vex repair migrate-home"));

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn test_exec_uses_project_toolchain_without_switching_global_state() {
    let home = fresh_temp_dir("vex_test_exec_home");
    let project = fresh_temp_dir("vex_test_exec_project");
    let global_bin = home.join(".vex/toolchains/node/18.0.0/bin");
    let project_bin = home.join(".vex/toolchains/node/20.11.0/bin");
    fs::create_dir_all(&global_bin).unwrap();
    fs::create_dir_all(&project_bin).unwrap();
    fs::create_dir_all(home.join(".vex/current")).unwrap();
    fs::create_dir_all(home.join(".vex/bin")).unwrap();
    fs::write(project.join(".tool-versions"), "node 20.11.0\n").unwrap();
    write_executable_script(
        &global_bin.join("node"),
        "#!/bin/sh\nprintf 'node-from-global:%s' \"$PWD\"\n",
    );
    write_executable_script(
        &project_bin.join("node"),
        "#!/bin/sh\nprintf 'node-from-exec:%s' \"$PWD\"\n",
    );
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        home.join(".vex/toolchains/node/18.0.0"),
        home.join(".vex/current/node"),
    )
    .unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        home.join(".vex/toolchains/node/18.0.0/bin/node"),
        home.join(".vex/bin/node"),
    )
    .unwrap();

    let output = vex_bin()
        .args(["exec", "--", "node"])
        .env("HOME", &home)
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("node-from-exec"));
    assert!(stdout.contains(project.to_string_lossy().as_ref()));
    let current = fs::read_link(home.join(".vex/current/node")).unwrap();
    assert_eq!(current, home.join(".vex/toolchains/node/18.0.0"));

    let _ = fs::remove_dir_all(&home);
    let _ = fs::remove_dir_all(&project);
}

#[test]
fn test_run_uses_vex_toml_command_env_and_project_root() {
    let home = fresh_temp_dir("vex_test_run_home");
    let project = fresh_temp_dir("vex_test_run_project");
    let nested = project.join("nested/deeper");
    fs::create_dir_all(&nested).unwrap();
    fs::create_dir_all(project.join(".venv/bin")).unwrap();
    let toolchain_bin = home.join(".vex/toolchains/node/20.11.0/bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::write(project.join(".tool-versions"), "node 20.11.0\n").unwrap();
    fs::write(
        project.join(".vex.toml"),
        r#"
[env]
APP_ENV = "dev"

[commands]
show = "node"
"#,
    )
    .unwrap();
    write_executable_script(
        &toolchain_bin.join("node"),
        "#!/bin/sh\nprintf '%s|%s|%s' \"$APP_ENV\" \"$VIRTUAL_ENV\" \"$PWD\"\n",
    );

    let output = vex_bin()
        .args(["run", "show"])
        .env("HOME", &home)
        .current_dir(&nested)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dev"));
    assert!(stdout.contains(project.join(".venv").to_string_lossy().as_ref()));
    assert!(stdout.contains(project.to_string_lossy().as_ref()));

    let _ = fs::remove_dir_all(&home);
    let _ = fs::remove_dir_all(&project);
}

#[test]
fn test_run_preserves_activation_path_when_shell_profile_mutates_path() {
    let home = fresh_temp_dir("vex_test_run_profile_home");
    let project = fresh_temp_dir("vex_test_run_profile_project");
    let nested = project.join("nested/deeper");
    fs::create_dir_all(&nested).unwrap();
    fs::create_dir_all(project.join(".venv/bin")).unwrap();
    let toolchain_bin = home.join(".vex/toolchains/node/20.11.0/bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::write(project.join(".tool-versions"), "node 20.11.0\n").unwrap();
    fs::write(
        project.join(".vex.toml"),
        r#"
[commands]
show = "node"
"#,
    )
    .unwrap();
    fs::write(home.join(".bash_profile"), "export PATH=/usr/bin:/bin\n").unwrap();
    write_executable_script(
        &toolchain_bin.join("node"),
        "#!/bin/sh\nprintf 'managed-node'\n",
    );

    let output = vex_bin()
        .args(["run", "show"])
        .env("HOME", &home)
        .env("SHELL", "/bin/bash")
        .current_dir(&nested)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("managed-node"), "stdout was: {}", stdout);

    let _ = fs::remove_dir_all(&home);
    let _ = fs::remove_dir_all(&project);
}

#[test]
fn test_relink_node_rebuilds_missing_dynamic_binary_link() {
    let home = fresh_temp_dir("vex_test_relink_home");
    let node_bin = home.join(".vex/toolchains/node/24.0.0/bin");
    fs::create_dir_all(home.join(".vex/current")).unwrap();
    fs::create_dir_all(home.join(".vex/bin")).unwrap();
    fs::create_dir_all(&node_bin).unwrap();

    for name in &["node", "npm", "npx"] {
        write_executable_script(&node_bin.join(name), "#!/bin/sh\nprintf 'ok'\n");
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(
        home.join(".vex/toolchains/node/24.0.0"),
        home.join(".vex/current/node"),
    )
    .unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        home.join(".vex/toolchains/node/24.0.0/bin/node"),
        home.join(".vex/bin/node"),
    )
    .unwrap();

    write_executable_script(&node_bin.join("openclaw"), "#!/bin/sh\nprintf 'openclaw'\n");

    let output = vex_bin()
        .args(["relink", "node"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert!(home.join(".vex/bin/openclaw").exists());

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn test_relink_rejects_unsupported_tool() {
    let home = fresh_temp_dir("vex_test_relink_unsupported");
    let output = vex_bin()
        .args(["relink", "python"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("supports node only"));

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn test_run_requires_project_task_definition() {
    let home = fresh_temp_dir("vex_test_run_missing_home");
    let project = fresh_temp_dir("vex_test_run_missing_project");

    let output = vex_bin()
        .args(["run", "show"])
        .env("HOME", &home)
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(".vex.toml"));

    let _ = fs::remove_dir_all(&home);
    let _ = fs::remove_dir_all(&project);
}

// --- 错误场景测试 ---

#[test]
fn test_invalid_tool() {
    let output = vex_bin().args(["list-remote", "ruby"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_use_nonexistent_version() {
    let home = fresh_temp_dir("vex_test_use_nonexistent");
    seed_remote_cache(&home, "node", &["20.11.0", "22.0.0"]);

    let output = vex_bin()
        .args(["use", "node@99.99.99"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Version not found"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_use_prefers_latest_installed_partial_match() {
    let home = fresh_temp_dir("vex_test_use_installed_partial");
    let latest_installed_bin = home.join(".vex/toolchains/node/20.20.1/bin");
    let older_installed_bin = home.join(".vex/toolchains/node/20.9.0/bin");
    fs::create_dir_all(&latest_installed_bin).unwrap();
    fs::create_dir_all(&older_installed_bin).unwrap();
    write_executable_script(
        &latest_installed_bin.join("node"),
        "#!/bin/sh\nprintf '20.20.1'\n",
    );
    write_executable_script(
        &older_installed_bin.join("node"),
        "#!/bin/sh\nprintf '20.9.0'\n",
    );
    seed_remote_cache(&home, "node", &["20.20.2", "20.20.1", "20.9.0"]);

    let output = vex_bin()
        .args(["use", "node@20"])
        .env("HOME", &home)
        .output()
        .unwrap();

    assert!(output.status.success(), "{:?}", output);
    let current = std::fs::read_link(home.join(".vex/current/node")).unwrap();
    assert_eq!(current, home.join(".vex/toolchains/node/20.20.1"));

    let _ = std::fs::remove_dir_all(&home);
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
fn test_list_installed_json() {
    let home = fresh_temp_dir("vex_test_list_json");
    let output = vex_bin()
        .args(["list", "node", "--json"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed.get("tool").unwrap(), "node");
    assert!(parsed.get("versions").unwrap().is_array());

    let _ = std::fs::remove_dir_all(&home);
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
    let output = vex_bin().args(["install", "ruby@3.0"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stderr.contains("Tool not found") || stdout.contains("Tool not found"));
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
    // All supported tools should be listable (even if not installed)
    for tool in &["node", "go", "java", "rust", "python"] {
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
    let home = fresh_temp_dir("vex_test_init_hint");
    let output = vex_bin().arg("init").env("HOME", &home).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("echo"));
    assert!(stdout.contains("vex env zsh"));

    let _ = std::fs::remove_dir_all(&home);
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
        .env("HOME", &temp_dir) // isolate from real ~/.vex/tool-versions
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
    let output = vex_bin().args(["local", "ruby@3.0"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_local_writes_tool_versions() {
    let home = fresh_temp_dir("vex_test_local_write_home");
    let dir = home.join("project");
    std::fs::create_dir_all(&dir).unwrap();
    seed_remote_cache(&home, "node", &["20.11.0"]);

    // local 命令会调用 resolve_fuzzy_version，对完整版本号直接返回
    let output = vex_bin()
        .args(["local", "node@20.11.0"])
        .env("HOME", &home)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(output.status.success());

    let tv = std::fs::read_to_string(dir.join(".tool-versions")).unwrap();
    assert!(tv.contains("node 20.11.0"));

    let _ = std::fs::remove_dir_all(&home);
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
    let output = vex_bin().args(["upgrade", "ruby"]).output().unwrap();
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

#[test]
fn test_outdated_invalid_tool() {
    let output = vex_bin().args(["outdated", "ruby"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_outdated_json_empty_context() {
    let home = fresh_temp_dir("vex_test_outdated_json");
    let output = vex_bin()
        .args(["outdated", "--json"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.get("scope").is_some());
    assert!(parsed.get("entries").unwrap().is_array());

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_prune_dry_run_empty_home() {
    let home = fresh_temp_dir("vex_test_prune_dry_run");
    let output = vex_bin()
        .args(["prune", "--dry-run"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Nothing to prune") || stdout.contains("candidate"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_gc_alias_dry_run() {
    let home = fresh_temp_dir("vex_test_gc_alias");
    let output = vex_bin()
        .args(["gc", "--dry-run"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_prune_removes_only_unmanaged_items() {
    let home = fresh_temp_dir("vex_test_prune_real_home");
    let project = fresh_temp_dir("vex_test_prune_real_project");

    fs::create_dir_all(home.join(".vex/cache")).unwrap();
    fs::create_dir_all(home.join(".vex/locks")).unwrap();
    fs::create_dir_all(home.join(".vex/current")).unwrap();
    fs::create_dir_all(home.join(".vex/bin")).unwrap();
    fs::create_dir_all(home.join(".vex/toolchains/node/1.0.0/bin")).unwrap();
    fs::create_dir_all(home.join(".vex/toolchains/node/2.0.0/bin")).unwrap();
    fs::create_dir_all(home.join(".vex/toolchains/node/3.0.0/bin")).unwrap();
    fs::write(home.join(".vex/tool-versions"), "node 1.0.0\n").unwrap();
    fs::write(project.join(".tool-versions"), "node 2.0.0\n").unwrap();
    fs::write(home.join(".vex/cache/remote-node.json"), "broken-json").unwrap();
    fs::write(home.join(".vex/locks/stale.lock"), "stale").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        home.join(".vex/toolchains/node/1.0.0"),
        home.join(".vex/current/node"),
    )
    .unwrap();

    let old_time = filetime::FileTime::from_unix_time(
        std::time::SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(60 * 60 * 2))
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        0,
    );
    filetime::set_file_mtime(home.join(".vex/locks/stale.lock"), old_time).unwrap();

    let output = vex_bin()
        .args(["prune"])
        .env("HOME", &home)
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);

    assert!(home.join(".vex/toolchains/node/1.0.0").exists());
    assert!(home.join(".vex/toolchains/node/2.0.0").exists());
    assert!(!home.join(".vex/toolchains/node/3.0.0").exists());
    assert!(!home.join(".vex/cache/remote-node.json").exists());
    assert!(!home.join(".vex/locks/stale.lock").exists());

    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&project);
}

// --- alias 命令测试 ---

#[test]
fn test_alias_list_no_aliases() {
    let output = vex_bin().args(["alias", "list"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No aliases found") || stdout.contains("aliases"));
}

#[test]
fn test_alias_set_invalid_tool() {
    let output = vex_bin()
        .args(["alias", "set", "ruby", "test", "3.0.0"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

// --- install from version files ---

#[test]
fn test_install_from_version_file_unsupported_tool() {
    let dir = std::env::temp_dir().join("vex_test_install_unsupported");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Write .tool-versions with a truly unsupported tool
    std::fs::write(dir.join(".tool-versions"), "ruby 3.2.0\n").unwrap();

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
    let output = vex_bin().args(["global", "ruby@3.0"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

#[test]
fn test_global_writes_tool_versions() {
    let home = fresh_temp_dir("vex_test_global_home");
    seed_remote_cache(&home, "node", &["20.11.0"]);

    let output = vex_bin()
        .args(["global", "node@20.11.0"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("node") && stdout.contains("20.11.0"));

    let tv = std::fs::read_to_string(home.join(".vex/tool-versions")).unwrap();
    assert!(tv.contains("node 20.11.0"));

    let _ = std::fs::remove_dir_all(&home);
}

// --- use --auto with version file ---

#[test]
fn test_use_auto_with_unsupported_tool() {
    let dir = std::env::temp_dir().join("vex_test_auto_unsupported");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(dir.join(".tool-versions"), "ruby 3.2.0\n").unwrap();

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

#[test]
fn test_doctor_json() {
    let home = fresh_temp_dir("vex_test_doctor_json");
    let output = vex_bin()
        .args(["doctor", "--json"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.get("root").is_some());
    assert!(parsed.get("checks").unwrap().is_array());

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_doctor_json_reports_invalid_effective_settings_from_project_config() {
    let home = fresh_temp_dir("vex_test_doctor_effective_settings_home");
    let project = fresh_temp_dir("vex_test_doctor_effective_settings_project");

    fs::create_dir_all(home.join(".vex/cache")).unwrap();
    fs::create_dir_all(home.join(".vex/locks")).unwrap();
    fs::create_dir_all(home.join(".vex/toolchains")).unwrap();
    fs::create_dir_all(home.join(".vex/current")).unwrap();
    fs::create_dir_all(home.join(".vex/bin")).unwrap();
    fs::write(home.join(".vex/config.toml"), "# valid config\n").unwrap();
    fs::write(
        project.join(".vex.toml"),
        r#"
[network]
proxy = "http:// bad proxy"

[mirrors]
node = "not-a-url"
"#,
    )
    .unwrap();

    let output = vex_bin()
        .args(["doctor", "--json"])
        .env("HOME", &home)
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();
    let checks = parsed
        .get("checks")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .find(|item| item.get("id") == Some(&Value::String("effective_settings".to_string())))
        .cloned()
        .unwrap();

    assert_eq!(
        checks.get("status"),
        Some(&Value::String("warn".to_string()))
    );
    let details = checks
        .get("details")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(details
        .iter()
        .any(|detail| detail.contains("Invalid proxy URL")));
    assert!(details
        .iter()
        .any(|detail| detail.contains("Invalid mirror for node")));

    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&project);
}

#[test]
fn test_doctor_json_reports_invalid_global_config_schema() {
    let home = fresh_temp_dir("vex_test_doctor_invalid_global_config");

    fs::create_dir_all(home.join(".vex/cache")).unwrap();
    fs::create_dir_all(home.join(".vex/locks")).unwrap();
    fs::create_dir_all(home.join(".vex/toolchains")).unwrap();
    fs::create_dir_all(home.join(".vex/current")).unwrap();
    fs::create_dir_all(home.join(".vex/bin")).unwrap();
    fs::write(
        home.join(".vex/config.toml"),
        r#"
[network]
connect_timeout_secs = "oops"
"#,
    )
    .unwrap();

    let output = vex_bin()
        .args(["doctor", "--json"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(&stdout).unwrap();
    let checks = parsed.get("checks").and_then(Value::as_array).unwrap();

    let config_check = checks
        .iter()
        .find(|item| item.get("id") == Some(&Value::String("config".to_string())))
        .cloned()
        .unwrap();
    assert_eq!(
        config_check.get("status"),
        Some(&Value::String("warn".to_string()))
    );
    let config_details = config_check
        .get("details")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(config_details
        .iter()
        .any(|detail| detail.contains("Failed to parse")));

    let effective_settings_check = checks
        .iter()
        .find(|item| item.get("id") == Some(&Value::String("effective_settings".to_string())))
        .cloned()
        .unwrap();
    assert_eq!(
        effective_settings_check.get("status"),
        Some(&Value::String("warn".to_string()))
    );
    let effective_details = effective_settings_check
        .get("details")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(effective_details
        .iter()
        .any(|detail| detail.contains("Failed to parse")));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_current_json_fails_on_invalid_global_config_schema() {
    let home = fresh_temp_dir("vex_test_current_invalid_global_config");

    fs::create_dir_all(home.join(".vex/current")).unwrap();
    fs::write(
        home.join(".vex/config.toml"),
        r#"
[network]
connect_timeout_secs = "oops"
"#,
    )
    .unwrap();

    let output = vex_bin()
        .args(["current", "--json"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(!output.status.success(), "{:?}", output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to parse"), "stderr was: {}", stderr);

    let _ = std::fs::remove_dir_all(&home);
}

// --- init 重复运行 ---

#[test]
fn test_init_idempotent() {
    let home = fresh_temp_dir("vex_test_init_idempotent");

    // Running init twice should succeed both times
    let output1 = vex_bin().arg("init").env("HOME", &home).output().unwrap();
    assert!(output1.status.success());
    let output2 = vex_bin().arg("init").env("HOME", &home).output().unwrap();
    assert!(output2.status.success());

    let _ = std::fs::remove_dir_all(&home);
}

// --- list-remote with --no-cache ---

#[test]
fn test_list_remote_no_cache_invalid_tool() {
    let output = vex_bin()
        .args(["list-remote", "ruby", "--no-cache"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Tool not found"));
}

// --- python subcommand tests ---

#[test]
fn test_python_unknown_subcmd() {
    let output = vex_bin().args(["python", "build"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown python subcommand"));
}

#[test]
fn test_python_freeze_no_venv() {
    let dir = std::env::temp_dir().join("vex_test_python_freeze_no_venv");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let output = vex_bin()
        .args(["python", "freeze"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No .venv found"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_python_sync_no_lock() {
    let dir = std::env::temp_dir().join("vex_test_python_sync_no_lock");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let output = vex_bin()
        .args(["python", "sync"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No requirements.lock found"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_python_base_requires_active_python() {
    let home = fresh_temp_dir("vex_test_python_base_no_active");
    let output = vex_bin()
        .args(["python", "base"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No active vex-managed Python"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_python_base_path_creates_base_env() {
    let home = fresh_temp_dir("vex_test_python_base_path");
    let toolchain = home.join(".vex/toolchains/python/3.13.3");
    let toolchain_bin = toolchain.join("bin");
    fs::create_dir_all(&toolchain_bin).unwrap();
    fs::create_dir_all(home.join(".vex/current")).unwrap();
    write_executable_script(
        &toolchain_bin.join("python3"),
        r#"#!/bin/sh
if [ "$1" = "-m" ] && [ "$2" = "venv" ]; then
  mkdir -p "$3/bin"
  printf '#!/bin/sh\n' > "$3/bin/python"
  printf '#!/bin/sh\n' > "$3/bin/pip"
  chmod +x "$3/bin/python" "$3/bin/pip"
  exit 0
fi
exit 42
"#,
    );
    std::os::unix::fs::symlink(&toolchain, home.join(".vex/current/python")).unwrap();

    let output = vex_bin()
        .args(["python", "base", "path"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let expected_base = home.join(".vex/python/base/3.13.3");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_base.display().to_string()
    );
    assert!(expected_base.join("bin/python").exists());
    assert!(expected_base.join("bin/pip").exists());

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_python_list_installed() {
    let output = vex_bin().args(["list", "python"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Installed versions") || stdout.contains("No versions"));
}

#[test]
fn test_install_from_version_file_python_skipped_or_handled() {
    let dir = std::env::temp_dir().join("vex_test_python_version_file");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(dir.join(".tool-versions"), "python 3.12.0\n").unwrap();

    let output = vex_bin().arg("install").current_dir(&dir).output().unwrap();
    // python is now a supported tool — should not produce "skipping unsupported tool"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("skipping unsupported tool 'python'"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_use_auto_with_python_version_file() {
    let dir = std::env::temp_dir().join("vex_test_auto_python");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(dir.join(".tool-versions"), "python 3.12.0\n").unwrap();

    let output = vex_bin()
        .args(["use", "--auto"])
        .current_dir(&dir)
        .output()
        .unwrap();
    // Should succeed (uninstalled python is warned, not a fatal error)
    assert!(output.status.success());

    let _ = std::fs::remove_dir_all(&dir);
}

// --- init shell configuration tests ---

#[test]
fn test_init_with_shell_auto() {
    let home = std::env::temp_dir().join("vex_test_init_auto");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let output = vex_bin()
        .args(["init", "--shell", "auto"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should either detect a shell or warn about inability to detect
    assert!(stdout.contains("Configured") || stdout.contains("Unable to detect"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_init_with_shell_zsh() {
    let home = std::env::temp_dir().join("vex_test_init_zsh");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let output = vex_bin()
        .args(["init", "--shell", "zsh"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check for "Configured" and "zsh" (may have ANSI codes)
    assert!(stdout.contains("Configured") && stdout.contains("zsh"));

    // Verify .zshrc was created and contains vex hook
    let zshrc = std::fs::read_to_string(home.join(".zshrc")).unwrap();
    assert!(zshrc.contains("vex env zsh"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_init_with_shell_bash() {
    let home = std::env::temp_dir().join("vex_test_init_bash");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let output = vex_bin()
        .args(["init", "--shell", "bash"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configured") && stdout.contains("bash"));

    // Verify .bashrc or .bash_profile was created
    let bashrc_exists = home.join(".bashrc").exists();
    let bash_profile_exists = home.join(".bash_profile").exists();
    assert!(bashrc_exists || bash_profile_exists);

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_init_with_shell_fish() {
    let home = std::env::temp_dir().join("vex_test_init_fish");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let output = vex_bin()
        .args(["init", "--shell", "fish"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configured") && stdout.contains("fish"));

    // Verify fish config was created
    let fish_config = home.join(".config/fish/config.fish");
    assert!(fish_config.exists());
    let content = std::fs::read_to_string(fish_config).unwrap();
    assert!(content.contains("vex env fish"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_init_with_dry_run_zsh() {
    let home = std::env::temp_dir().join("vex_test_init_dry_run");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let output = vex_bin()
        .args(["init", "--shell", "zsh", "--dry-run"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Would") || stdout.contains("Preview"));

    // Verify .zshrc was NOT created
    assert!(!home.join(".zshrc").exists());

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_init_with_shell_skip_manual() {
    let home = std::env::temp_dir().join("vex_test_init_skip");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let output = vex_bin()
        .args(["init", "--shell", "skip"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vex init --shell auto") || stdout.contains("manually configure"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_init_already_configured_zsh() {
    let home = std::env::temp_dir().join("vex_test_init_already");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    // Pre-create .zshrc with vex hook
    std::fs::write(
        home.join(".zshrc"),
        "# existing config\neval \"$(vex env zsh)\"\n",
    )
    .unwrap();

    let output = vex_bin()
        .args(["init", "--shell", "zsh"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("already") && stdout.contains("configured"));

    // Verify .zshrc was not modified (no duplicate hook)
    let zshrc = std::fs::read_to_string(home.join(".zshrc")).unwrap();
    assert_eq!(zshrc.matches("vex env zsh").count(), 1);

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_init_unsupported_shell_powershell() {
    let home = std::env::temp_dir().join("vex_test_init_unsupported");
    let _ = std::fs::remove_dir_all(&home);
    std::fs::create_dir_all(&home).unwrap();

    let output = vex_bin()
        .args(["init", "--shell", "powershell"])
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(output.status.success()); // Should not fail, just warn
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Check both stdout and stderr for the error message
    assert!(stdout.contains("Unsupported shell") || stderr.contains("Unsupported shell"));

    let _ = std::fs::remove_dir_all(&home);
}

// --- install --no-switch 测试 ---

#[test]
fn test_install_no_switch_flag() {
    let home = fresh_temp_dir("vex_test_install_no_switch");
    std::fs::create_dir_all(home.join(".vex/toolchains/node/20.11.0")).unwrap();
    seed_remote_cache(&home, "node", &["20.11.0"]);

    let output = vex_bin()
        .args(["install", "node@20.11.0", "--no-switch"])
        .env("HOME", &home)
        .output()
        .unwrap();

    assert!(output.status.success());

    // 命令应该成功（即使版本未安装，也会显示提示）
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 已安装路径不需要联网，也应该给出可切换的提示
    assert!(
        stdout.contains("already installed")
            || stdout.contains("Use 'vex use node@20.11.0' to switch to it.")
            || stderr.contains("already installed"),
        "Should show already-installed or switch guidance"
    );

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_install_help_shows_no_switch() {
    let output = vex_bin().args(["install", "--help"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--no-switch"),
        "Help should document --no-switch flag"
    );
}

// --- Multi-spec install tests ---

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_install_multiple_specs() {
    let home = fresh_temp_dir("vex_test_multi_install");

    // Create fake installations to avoid network calls
    std::fs::create_dir_all(home.join(".vex/toolchains/node/20.11.0")).unwrap();
    std::fs::create_dir_all(home.join(".vex/toolchains/go/1.23.5")).unwrap();

    let output = vex_bin()
        .args(["install", "node@20.11.0", "go@1.23.5"])
        .env("HOME", &home)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Installation Summary") || stdout.contains("already installed"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_install_multiple_specs_with_invalid() {
    let home = fresh_temp_dir("vex_test_multi_invalid");

    let output = vex_bin()
        .args(["install", "node@20.11.0", "ruby@3.0"])
        .env("HOME", &home)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("Tool not found") || stderr.contains("Tool not found"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_install_from_flag() {
    let home = fresh_temp_dir("vex_test_install_from");
    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir).unwrap();

    // Create a .tool-versions file
    std::fs::write(
        project_dir.join(".tool-versions"),
        "node 20.11.0\ngo 1.23.5\n",
    )
    .unwrap();

    // Create fake installations
    std::fs::create_dir_all(home.join(".vex/toolchains/node/20.11.0")).unwrap();
    std::fs::create_dir_all(home.join(".vex/toolchains/go/1.23.5")).unwrap();

    let output = vex_bin()
        .args(["install", "--from", ".tool-versions"])
        .env("HOME", &home)
        .current_dir(&project_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Sync Summary") || stdout.contains("already installed"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_install_from_team_config_file_prefers_local_tool_versions() {
    let home = fresh_temp_dir("vex_test_install_from_team_config");
    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir).unwrap();

    std::fs::write(
        project_dir.join("vex-config.toml"),
        "version = 1\n\n[tools]\nnode = \"20.12.2\"\n",
    )
    .unwrap();
    std::fs::write(project_dir.join(".tool-versions"), "node 22.0.0\n").unwrap();
    std::fs::create_dir_all(home.join(".vex/toolchains/node/22.0.0")).unwrap();

    let output = vex_bin()
        .args(["install", "--from", "vex-config.toml"])
        .env("HOME", &home)
        .current_dir(&project_dir)
        .output()
        .unwrap();

    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("22.0.0"));
    assert!(stdout.contains("already installed"));

    let _ = std::fs::remove_dir_all(&home);
}

// --- Sync command tests ---

#[test]
fn test_sync_command() {
    let home = fresh_temp_dir("vex_test_sync");
    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir).unwrap();

    // Create a .tool-versions file
    std::fs::write(project_dir.join(".tool-versions"), "node 20.11.0\n").unwrap();

    // Create fake installation
    std::fs::create_dir_all(home.join(".vex/toolchains/node/20.11.0")).unwrap();

    let output = vex_bin()
        .args(["sync"])
        .env("HOME", &home)
        .current_dir(&project_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Sync Summary") || stdout.contains("already installed"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_sync_no_version_file() {
    let home = fresh_temp_dir("vex_test_sync_no_file");
    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir).unwrap();

    let output = vex_bin()
        .args(["sync"])
        .env("HOME", &home)
        .current_dir(&project_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No version files found"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_sync_from_flag() {
    let home = fresh_temp_dir("vex_test_sync_from");
    let project_dir = home.join("project");
    std::fs::create_dir_all(&project_dir).unwrap();

    // Create a custom version file
    std::fs::write(project_dir.join("custom-versions.txt"), "node 20.11.0\n").unwrap();

    // Create fake installation
    std::fs::create_dir_all(home.join(".vex/toolchains/node/20.11.0")).unwrap();

    let output = vex_bin()
        .args(["sync", "--from", "custom-versions.txt"])
        .env("HOME", &home)
        .current_dir(&project_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Sync Summary") || stdout.contains("already installed"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_sync_help() {
    let output = vex_bin().args(["sync", "--help"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Install missing versions"));
}

// Advisory warning tests - verify that lifecycle warnings are shown

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_install_shows_eol_warning() {
    // This test verifies that installing an EOL version shows advisory warning
    // Example: node@16 is EOL and should show warning
    let home = fresh_temp_dir("vex_test_install_eol_warning");

    // Initialize vex
    let _ = vex_bin()
        .args(["init", "--shell", "zsh"])
        .env("HOME", &home)
        .output()
        .unwrap();

    // Install EOL version
    let output = vex_bin()
        .args(["install", "node@16"])
        .env("HOME", &home)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Should show warning symbol and EOL message
    assert!(
        combined.contains("⚠") || combined.contains("warning") || combined.contains("end-of-life"),
        "Expected EOL warning in output, got: {}",
        combined
    );

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_use_shows_eol_warning() {
    // This test verifies that using an EOL version shows advisory warning
    let home = fresh_temp_dir("vex_test_use_eol_warning");

    // Initialize and install EOL version
    let _ = vex_bin()
        .args(["init", "--shell", "zsh"])
        .env("HOME", &home)
        .output()
        .unwrap();

    let _ = vex_bin()
        .args(["install", "node@16", "--no-switch"])
        .env("HOME", &home)
        .output()
        .unwrap();

    // Use the EOL version
    let output = vex_bin()
        .args(["use", "node@16"])
        .env("HOME", &home)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Should show warning and recommendation
    assert!(
        combined.contains("warning:") || combined.contains("end-of-life"),
        "Expected EOL warning in output, got: {}",
        combined
    );
    assert!(
        combined.contains("recommendation:") || combined.contains("upgrade"),
        "Expected recommendation in output, got: {}",
        combined
    );

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
#[cfg_attr(
    not(feature = "network-tests"),
    ignore = "requires --features network-tests"
)]
fn test_install_from_file_shows_warnings() {
    // This test verifies that install --from shows advisory warnings
    let home = fresh_temp_dir("vex_test_install_from_file_warnings");
    let project_dir = home.join("project");
    fs::create_dir_all(&project_dir).unwrap();

    // Create .tool-versions with EOL version
    fs::write(project_dir.join(".tool-versions"), "node 16.20.2\n").unwrap();

    // Initialize vex
    let _ = vex_bin()
        .args(["init", "--shell", "zsh"])
        .env("HOME", &home)
        .output()
        .unwrap();

    // Install from file
    let output = vex_bin()
        .args(["install", "--from", ".tool-versions"])
        .env("HOME", &home)
        .current_dir(&project_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show warning in summary
    assert!(
        stdout.contains("⚠") || stdout.contains("end-of-life"),
        "Expected EOL warning in install summary, got: {}",
        stdout
    );

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_lock_command_without_version_file() {
    let home = fresh_temp_dir("vex_test_lock_no_version");
    let project_dir = home.join("project");
    fs::create_dir_all(&project_dir).unwrap();

    // Initialize vex
    let _ = vex_bin()
        .args(["init"])
        .env("HOME", &home)
        .output()
        .unwrap();

    // Try to generate lockfile without .tool-versions
    let output = vex_bin()
        .arg("lock")
        .env("HOME", &home)
        .current_dir(&project_dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No version files found"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_lock_command_with_uninstalled_version() {
    let home = fresh_temp_dir("vex_test_lock_uninstalled");
    let project_dir = home.join("project");
    fs::create_dir_all(&project_dir).unwrap();

    // Initialize vex
    let _ = vex_bin()
        .args(["init"])
        .env("HOME", &home)
        .output()
        .unwrap();

    // Create .tool-versions with uninstalled version
    fs::write(project_dir.join(".tool-versions"), "node 20.11.0\n").unwrap();

    // Try to generate lockfile
    let output = vex_bin()
        .arg("lock")
        .env("HOME", &home)
        .current_dir(&project_dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not installed"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_install_frozen_without_lockfile() {
    let home = fresh_temp_dir("vex_test_frozen_no_lock");
    let project_dir = home.join("project");
    fs::create_dir_all(&project_dir).unwrap();

    // Initialize vex
    let _ = vex_bin()
        .args(["init"])
        .env("HOME", &home)
        .output()
        .unwrap();

    // Create .tool-versions
    fs::write(project_dir.join(".tool-versions"), "node 20.11.0\n").unwrap();

    // Try frozen install without lockfile
    let output = vex_bin()
        .args(["install", "--frozen"])
        .env("HOME", &home)
        .current_dir(&project_dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Frozen mode requires a lockfile"));

    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn test_sync_frozen_without_lockfile() {
    let home = fresh_temp_dir("vex_test_sync_frozen_no_lock");
    let project_dir = home.join("project");
    fs::create_dir_all(&project_dir).unwrap();

    // Initialize vex
    let _ = vex_bin()
        .args(["init"])
        .env("HOME", &home)
        .output()
        .unwrap();

    // Create .tool-versions
    fs::write(project_dir.join(".tool-versions"), "node 20.11.0\n").unwrap();

    // Try frozen sync without lockfile
    let output = vex_bin()
        .args(["sync", "--frozen"])
        .env("HOME", &home)
        .current_dir(&project_dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Frozen mode requires a lockfile"));

    let _ = std::fs::remove_dir_all(&home);
}
