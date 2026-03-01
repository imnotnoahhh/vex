//! 端到端集成测试
//!
//! 测试完整的工作流程：安装 → 切换 → 卸载
//! 注意：这些测试需要网络连接和较长的运行时间

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn vex_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vex"))
}

fn vex_home() -> PathBuf {
    dirs::home_dir().unwrap().join(".vex")
}

/// 测试辅助函数：检查版本是否已安装
fn is_installed(tool: &str, version: &str) -> bool {
    vex_home()
        .join("toolchains")
        .join(tool)
        .join(version)
        .exists()
}

/// 测试辅助函数：检查版本是否已激活
fn is_active(tool: &str) -> bool {
    vex_home().join("current").join(tool).exists()
}

// --- Node.js 端到端测试 ---

#[test]
#[ignore] // 需要网络和较长时间
fn test_e2e_node_install_use_uninstall() {
    let test_version = "20.11.0";

    // 1. 安装
    println!("Testing Node.js {} installation...", test_version);
    let output = vex_bin()
        .args(["install", &format!("node@{}", test_version)])
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // 如果已安装，跳过安装步骤
        if !stderr.contains("already installed") {
            panic!("Install failed: {}", stderr);
        }
    }

    assert!(
        is_installed("node", test_version),
        "Node.js {} should be installed",
        test_version
    );

    // 2. 切换版本
    println!("Testing Node.js {} activation...", test_version);
    let output = vex_bin()
        .args(["use", &format!("node@{}", test_version)])
        .output()
        .unwrap();
    assert!(output.status.success(), "Use command should succeed");
    assert!(is_active("node"), "Node.js should be active");

    // 3. 验证 current 链接
    let current_link = vex_home().join("current").join("node");
    assert!(current_link.exists(), "Current symlink should exist");
    let target = fs::read_link(&current_link).unwrap();
    assert!(
        target.to_string_lossy().contains(test_version),
        "Current symlink should point to {}",
        test_version
    );

    // 4. 验证 bin 链接
    let node_bin = vex_home().join("bin").join("node");
    assert!(node_bin.exists(), "node binary symlink should exist");

    // 5. 卸载
    println!("Testing Node.js {} uninstallation...", test_version);
    let output = vex_bin()
        .args(["uninstall", &format!("node@{}", test_version)])
        .output()
        .unwrap();
    assert!(output.status.success(), "Uninstall should succeed");
    assert!(
        !is_installed("node", test_version),
        "Node.js {} should be uninstalled",
        test_version
    );
}

// --- Go 端到端测试 ---

#[test]
#[ignore] // 需要网络和较长时间
fn test_e2e_go_install_use_uninstall() {
    let test_version = "1.22.0";

    // 1. 安装
    println!("Testing Go {} installation...", test_version);
    let output = vex_bin()
        .args(["install", &format!("go@{}", test_version)])
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("already installed") {
            panic!("Install failed: {}", stderr);
        }
    }

    assert!(
        is_installed("go", test_version),
        "Go {} should be installed",
        test_version
    );

    // 2. 切换版本
    println!("Testing Go {} activation...", test_version);
    let output = vex_bin()
        .args(["use", &format!("go@{}", test_version)])
        .output()
        .unwrap();
    assert!(output.status.success(), "Use command should succeed");
    assert!(is_active("go"), "Go should be active");

    // 3. 验证 bin 链接
    let go_bin = vex_home().join("bin").join("go");
    assert!(go_bin.exists(), "go binary symlink should exist");
    let gofmt_bin = vex_home().join("bin").join("gofmt");
    assert!(gofmt_bin.exists(), "gofmt binary symlink should exist");

    // 4. 卸载
    println!("Testing Go {} uninstallation...", test_version);
    let output = vex_bin()
        .args(["uninstall", &format!("go@{}", test_version)])
        .output()
        .unwrap();
    assert!(output.status.success(), "Uninstall should succeed");
    assert!(
        !is_installed("go", test_version),
        "Go {} should be uninstalled",
        test_version
    );
}

// --- 版本别名测试 ---

#[test]
#[ignore] // 需要网络
fn test_e2e_install_with_alias() {
    // 测试使用 lts 别名安装 Node.js
    println!("Testing Node.js LTS alias installation...");
    let output = vex_bin().args(["install", "node@lts"]).output().unwrap();

    // 应该成功或已安装
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("already installed"),
            "Install with alias should succeed or be already installed"
        );
    }
}

// --- 多版本切换测试 ---

#[test]
#[ignore] // 需要网络和较长时间
fn test_e2e_switch_between_versions() {
    let version1 = "20.11.0";
    let version2 = "20.10.0";

    // 安装两个版本
    for version in &[version1, version2] {
        let output = vex_bin()
            .args(["install", &format!("node@{}", version)])
            .output()
            .unwrap();
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("already installed") {
                panic!("Install node@{} failed: {}", version, stderr);
            }
        }
    }

    // 切换到版本1
    let output = vex_bin()
        .args(["use", &format!("node@{}", version1)])
        .output()
        .unwrap();
    assert!(output.status.success());

    let current_link = vex_home().join("current").join("node");
    let target1 = fs::read_link(&current_link).unwrap();
    assert!(target1.to_string_lossy().contains(version1));

    // 切换到版本2
    let output = vex_bin()
        .args(["use", &format!("node@{}", version2)])
        .output()
        .unwrap();
    assert!(output.status.success());

    let target2 = fs::read_link(&current_link).unwrap();
    assert!(target2.to_string_lossy().contains(version2));

    // 清理
    for version in &[version1, version2] {
        let _ = vex_bin()
            .args(["uninstall", &format!("node@{}", version)])
            .output();
    }
}

// --- .tool-versions 文件测试 ---

#[test]
fn test_e2e_tool_versions_file() {
    let temp_dir = TempDir::new().unwrap();
    let tool_versions = temp_dir.path().join(".tool-versions");

    // 创建 .tool-versions 文件
    fs::write(&tool_versions, "node 20.11.0\ngo 1.22.0\n").unwrap();

    // 测试 use --auto 能读取文件
    let output = vex_bin()
        .args(["use", "--auto"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // 应该成功（即使版本未安装，也不应该报错）
    assert!(output.status.success());
}

// --- local/global 命令测试 ---

#[test]
fn test_e2e_local_command() {
    let temp_dir = TempDir::new().unwrap();

    // 执行 local 命令
    let output = vex_bin()
        .args(["local", "node@20.11.0"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "local command should succeed");

    // 验证 .tool-versions 文件已创建
    let tool_versions = temp_dir.path().join(".tool-versions");
    assert!(tool_versions.exists(), ".tool-versions should be created");

    let content = fs::read_to_string(&tool_versions).unwrap();
    assert!(
        content.contains("node 20.11.0"),
        ".tool-versions should contain node 20.11.0"
    );
}

#[test]
fn test_e2e_global_command() {
    let global_versions = dirs::home_dir().unwrap().join(".tool-versions");

    // 备份现有的全局配置（如果存在）
    let backup = if global_versions.exists() {
        Some(fs::read_to_string(&global_versions).unwrap())
    } else {
        None
    };

    // 执行 global 命令
    let output = vex_bin()
        .args(["global", "node@20.11.0"])
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("global command failed: {}", stderr);
    }
    assert!(output.status.success(), "global command should succeed");

    // 验证全局配置文件已更新
    if !global_versions.exists() {
        eprintln!("Expected file at: {}", global_versions.display());
    }
    assert!(
        global_versions.exists(),
        "global .tool-versions should exist"
    );
    let content = fs::read_to_string(&global_versions).unwrap();
    assert!(
        content.contains("node 20.11.0"),
        "global .tool-versions should contain node 20.11.0"
    );

    // 恢复备份
    if let Some(backup_content) = backup {
        fs::write(&global_versions, backup_content).unwrap();
    } else {
        let _ = fs::remove_file(&global_versions);
    }
}

// --- list 命令测试 ---

#[test]
fn test_e2e_list_command() {
    // 测试 list 命令显示已安装版本
    let output = vex_bin().args(["list", "node"]).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 应该显示 "Installed versions" 或 "No versions"
    assert!(
        stdout.contains("Installed versions") || stdout.contains("No versions"),
        "list should show installation status"
    );
}

#[test]
#[ignore] // 需要网络
fn test_e2e_list_remote_command() {
    // 测试 list-remote 命令
    let output = vex_bin().args(["list-remote", "node"]).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 应该显示版本列表
    assert!(
        stdout.contains("Available versions") || stdout.contains("v"),
        "list-remote should show available versions"
    );
}

// --- current 命令测试 ---

#[test]
fn test_e2e_current_command() {
    let output = vex_bin().arg("current").output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 应该显示当前激活的版本或提示
    assert!(
        stdout.contains("active versions") || stdout.contains("No tools activated"),
        "current should show activation status"
    );
}

// --- 并发安装保护测试 ---

#[test]
#[ignore] // 需要网络和较长时间
fn test_e2e_concurrent_install_protection() {
    use std::thread;

    let version = "20.11.0";

    // 启动两个并发安装
    let handle1 = thread::spawn(move || {
        vex_bin()
            .args(["install", &format!("node@{}", version)])
            .output()
    });

    let handle2 = thread::spawn(move || {
        vex_bin()
            .args(["install", &format!("node@{}", version)])
            .output()
    });

    let result1 = handle1.join().unwrap().unwrap();
    let result2 = handle2.join().unwrap().unwrap();

    // 至少有一个应该成功或提示已安装
    let success_count = [result1, result2]
        .iter()
        .filter(|r| {
            r.status.success()
                || String::from_utf8_lossy(&r.stderr).contains("already installed")
                || String::from_utf8_lossy(&r.stderr).contains("Another vex process")
        })
        .count();

    assert!(
        success_count >= 1,
        "At least one install should succeed or be protected"
    );
}
