use crate::error::{Result, VexError};
use crate::tools::Tool;
use owo_colors::OwoColorize;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

fn vex_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".vex")
}

pub fn switch_version(tool: &dyn Tool, version: &str) -> Result<()> {
    switch_version_in(tool, version, &vex_dir())
}

fn switch_version_in(tool: &dyn Tool, version: &str, base_dir: &Path) -> Result<()> {
    let toolchain_dir = base_dir.join("toolchains").join(tool.name()).join(version);

    if !toolchain_dir.exists() {
        return Err(VexError::VersionNotFound {
            tool: tool.name().to_string(),
            version: version.to_string(),
        });
    }

    println!(
        "{} {} to version {}...",
        "Switching".cyan(),
        tool.name().yellow(),
        version.yellow()
    );

    // 1. 更新 current/ 符号链接
    let current_dir = base_dir.join("current");
    fs::create_dir_all(&current_dir)?;

    let current_link = current_dir.join(tool.name());
    let temp_link = current_link.with_extension("tmp");

    let _ = fs::remove_file(&temp_link);
    unix_fs::symlink(&toolchain_dir, &temp_link)?;
    fs::rename(&temp_link, &current_link)?;

    // 2. 更新 bin/ 下的可执行文件链接
    let bin_dir = base_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;

    for (bin_name, subpath) in tool.bin_paths() {
        let bin_link = bin_dir.join(bin_name);
        let target = toolchain_dir.join(subpath).join(bin_name);

        let _ = fs::remove_file(&bin_link);
        unix_fs::symlink(&target, &bin_link)?;
    }

    println!(
        "{} Switched {} to version {}",
        "✓".green(),
        tool.name().yellow(),
        version.yellow()
    );
    println!();
    let verify_flag = if tool.name() == "go" {
        "version"
    } else {
        "--version"
    };
    println!(
        "{} {}",
        "Verify with:".dimmed(),
        format!("{} {}", tool.bin_names()[0], verify_flag).cyan()
    );
    println!(
        "{} {}",
        "Note:".dimmed(),
        "If 'which' shows old paths, run: hash -r".dimmed()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::go::GoTool;
    use crate::tools::node::NodeTool;
    use crate::tools::rust::RustTool;

    fn make_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vex_switcher_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_switch_version_not_found() {
        let base = make_temp_dir("not_found");
        let result = switch_version_in(&NodeTool, "99.99.99", &base);
        assert!(result.is_err());
        if let Err(VexError::VersionNotFound { tool, version }) = result {
            assert_eq!(tool, "node");
            assert_eq!(version, "99.99.99");
        } else {
            panic!("Expected VersionNotFound error");
        }
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_creates_current_symlink() {
        let base = make_temp_dir("current_link");

        // 创建假的 toolchain 目录
        let tc = base.join("toolchains/node/20.11.0/bin");
        fs::create_dir_all(&tc).unwrap();
        // 创建假的二进制文件
        for name in &["node", "npm", "npx"] {
            fs::write(tc.join(name), "fake").unwrap();
        }

        let result = switch_version_in(&NodeTool, "20.11.0", &base);
        assert!(result.is_ok());

        // 验证 current/node 符号链接存在且指向正确
        let current_link = base.join("current/node");
        assert!(current_link.exists());
        let target = fs::read_link(&current_link).unwrap();
        assert!(target.ends_with("toolchains/node/20.11.0"));

        // 验证 bin/ 下的链接
        assert!(base.join("bin/node").exists());
        assert!(base.join("bin/npm").exists());
        assert!(base.join("bin/npx").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_go_creates_correct_links() {
        let base = make_temp_dir("go_links");

        let tc = base.join("toolchains/go/1.23.5/bin");
        fs::create_dir_all(&tc).unwrap();
        fs::write(tc.join("go"), "fake").unwrap();
        fs::write(tc.join("gofmt"), "fake").unwrap();

        let result = switch_version_in(&GoTool, "1.23.5", &base);
        assert!(result.is_ok());

        assert!(base.join("bin/go").exists());
        assert!(base.join("bin/gofmt").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_rust_separate_bin_paths() {
        let base = make_temp_dir("rust_paths");

        // Rust 的各组件在不同子目录
        let rustc_dir = base.join("toolchains/rust/1.93.1/rustc/bin");
        let cargo_dir = base.join("toolchains/rust/1.93.1/cargo/bin");
        let rustfmt_dir = base.join("toolchains/rust/1.93.1/rustfmt-preview/bin");
        let clippy_dir = base.join("toolchains/rust/1.93.1/clippy-preview/bin");
        let analyzer_dir = base.join("toolchains/rust/1.93.1/rust-analyzer-preview/bin");
        fs::create_dir_all(&rustc_dir).unwrap();
        fs::create_dir_all(&cargo_dir).unwrap();
        fs::create_dir_all(&rustfmt_dir).unwrap();
        fs::create_dir_all(&clippy_dir).unwrap();
        fs::create_dir_all(&analyzer_dir).unwrap();
        fs::write(rustc_dir.join("rustc"), "fake").unwrap();
        fs::write(rustc_dir.join("rustdoc"), "fake").unwrap();
        fs::write(rustc_dir.join("rust-gdb"), "fake").unwrap();
        fs::write(rustc_dir.join("rust-gdbgui"), "fake").unwrap();
        fs::write(rustc_dir.join("rust-lldb"), "fake").unwrap();
        fs::write(cargo_dir.join("cargo"), "fake").unwrap();
        fs::write(rustfmt_dir.join("rustfmt"), "fake").unwrap();
        fs::write(rustfmt_dir.join("cargo-fmt"), "fake").unwrap();
        fs::write(clippy_dir.join("cargo-clippy"), "fake").unwrap();
        fs::write(clippy_dir.join("clippy-driver"), "fake").unwrap();
        fs::write(analyzer_dir.join("rust-analyzer"), "fake").unwrap();

        let result = switch_version_in(&RustTool, "1.93.1", &base);
        assert!(result.is_ok());

        // 验证 bin 链接指向不同的子目录
        let rustc_target = fs::read_link(base.join("bin/rustc")).unwrap();
        let cargo_target = fs::read_link(base.join("bin/cargo")).unwrap();
        assert!(rustc_target.to_string_lossy().contains("rustc/bin/rustc"));
        assert!(cargo_target.to_string_lossy().contains("cargo/bin/cargo"));

        // 验证 clippy、rustfmt、rust-analyzer 链接
        assert!(base.join("bin/rustfmt").exists());
        assert!(base.join("bin/cargo-fmt").exists());
        assert!(base.join("bin/cargo-clippy").exists());
        assert!(base.join("bin/clippy-driver").exists());
        assert!(base.join("bin/rust-analyzer").exists());
        assert!(base.join("bin/rustdoc").exists());
        assert!(base.join("bin/rust-gdb").exists());
        assert!(base.join("bin/rust-gdbgui").exists());
        assert!(base.join("bin/rust-lldb").exists());

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn test_switch_replaces_existing_links() {
        let base = make_temp_dir("replace_links");

        // 先创建 v1
        let tc_v1 = base.join("toolchains/node/1.0.0/bin");
        fs::create_dir_all(&tc_v1).unwrap();
        for name in &["node", "npm", "npx"] {
            fs::write(tc_v1.join(name), "v1").unwrap();
        }
        switch_version_in(&NodeTool, "1.0.0", &base).unwrap();

        // 再切换到 v2
        let tc_v2 = base.join("toolchains/node/2.0.0/bin");
        fs::create_dir_all(&tc_v2).unwrap();
        for name in &["node", "npm", "npx"] {
            fs::write(tc_v2.join(name), "v2").unwrap();
        }
        switch_version_in(&NodeTool, "2.0.0", &base).unwrap();

        // current 应该指向 v2
        let target = fs::read_link(base.join("current/node")).unwrap();
        assert!(target.ends_with("toolchains/node/2.0.0"));

        let _ = fs::remove_dir_all(&base);
    }
}
