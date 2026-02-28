use crate::downloader::{download_with_retry, verify_checksum};
use crate::error::{Result, VexError};
use crate::lock::InstallLock;
use crate::tools::{Arch, Tool};
use flate2::read::GzDecoder;
use std::fs;
use std::path::PathBuf;
use tar::Archive;

fn vex_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".vex")
}

/// 清理守卫：在安装失败时自动清理临时文件
struct CleanupGuard {
    paths: Vec<PathBuf>,
    disarmed: bool,
}

impl CleanupGuard {
    fn new() -> Self {
        Self {
            paths: Vec::new(),
            disarmed: false,
        }
    }

    fn add(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    fn disarm(&mut self) {
        self.disarmed = true;
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if self.disarmed {
            return;
        }
        for path in &self.paths {
            if path.is_dir() {
                let _ = fs::remove_dir_all(path);
            } else if path.is_file() {
                let _ = fs::remove_file(path);
            }
        }
    }
}

pub fn install(tool: &dyn Tool, version: &str) -> Result<()> {
    let arch = Arch::detect();
    let vex = vex_dir();

    // 0. 检查是否已安装
    let final_dir = vex.join("toolchains").join(tool.name()).join(version);
    if final_dir.exists() {
        println!("{} {} is already installed.", tool.name(), version);
        println!("Use 'vex use {}@{}' to switch to it.", tool.name(), version);
        return Ok(());
    }

    // Acquire install lock (fail fast if another process is installing the same version)
    let _lock = InstallLock::acquire(&vex, tool.name(), version)?;

    println!("Installing {} {}...", tool.name(), version);

    let cache_dir = vex.join("cache");
    fs::create_dir_all(&cache_dir)?;

    let archive_name = format!("{}-{}.tar.gz", tool.name(), version);
    let archive_path = cache_dir.join(&archive_name);
    let extract_dir = cache_dir.join(format!("{}-{}-extract", tool.name(), version));

    // 设置清理守卫
    let mut guard = CleanupGuard::new();
    guard.add(archive_path.clone());
    guard.add(extract_dir.clone());

    // 1. 下载
    let download_url = tool.download_url(version, arch)?;
    println!("Downloading from {}...", download_url);
    download_with_retry(&download_url, &archive_path, 3)?;

    // 2. 验证 checksum
    if let Ok(Some(expected)) = tool.get_checksum(version, arch) {
        println!("Verifying checksum...");
        verify_checksum(&archive_path, &expected)?;
        println!("✓ Checksum verified");
    }

    // 3. 解压
    println!("Extracting...");
    fs::create_dir_all(&extract_dir)?;

    let tar_gz = fs::File::open(&archive_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(&extract_dir)?;

    // 4. 找到解压后的目录
    let entries = fs::read_dir(&extract_dir)?;
    let extracted_dir = entries
        .filter_map(|e| e.ok())
        .find(|e| e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false))
        .ok_or_else(|| VexError::Parse("No directory found after extraction".to_string()))?
        .path();

    // 5. 移动到最终位置
    let toolchains_dir = vex.join("toolchains").join(tool.name());
    fs::create_dir_all(&toolchains_dir)?;
    fs::rename(&extracted_dir, &final_dir)?;

    // 5.5. 运行 post-install 钩子
    tool.post_install(&final_dir, arch)?;

    // 6. 安装成功，解除清理守卫并手动清理临时文件
    guard.disarm();
    let _ = fs::remove_file(&archive_path);
    let _ = fs::remove_dir_all(&extract_dir);

    println!(
        "✓ Installed {} {} to {}",
        tool.name(),
        version,
        final_dir.display()
    );

    Ok(())
}
