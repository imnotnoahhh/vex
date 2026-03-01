use crate::downloader::{download_with_retry, verify_checksum};
use crate::error::{Result, VexError};
use crate::lock::InstallLock;
use crate::tools::{Arch, Tool};
use flate2::read::GzDecoder;
use owo_colors::OwoColorize;
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::Disks;
use tar::Archive;

/// Minimum required free space in bytes (500 MB)
const MIN_FREE_SPACE_BYTES: u64 = 500 * 1024 * 1024;

fn vex_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".vex")
}

/// Check if there is enough disk space available
fn check_disk_space(path: &Path, required_bytes: u64) -> Result<()> {
    let disks = Disks::new_with_refreshed_list();

    // Find the disk that contains the path
    for disk in &disks {
        if path.starts_with(disk.mount_point()) {
            let available = disk.available_space();
            if available < required_bytes {
                return Err(VexError::DiskSpace {
                    need: required_bytes / (1024 * 1024 * 1024),
                    available: available / (1024 * 1024 * 1024),
                });
            }
            return Ok(());
        }
    }

    // If we can't find the disk, proceed anyway (better than failing)
    Ok(())
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
        println!(
            "{} {} is already installed.",
            format!("{}@{}", tool.name(), version).yellow(),
            "✓".green()
        );
        println!(
            "Use {} to switch to it.",
            format!("'vex use {}@{}'", tool.name(), version).cyan()
        );
        return Ok(());
    }

    // Acquire install lock (fail fast if another process is installing the same version)
    let _lock = InstallLock::acquire(&vex, tool.name(), version)?;

    // Check disk space before downloading
    check_disk_space(&vex, MIN_FREE_SPACE_BYTES)?;

    println!(
        "{} {} {}...",
        "Installing".cyan(),
        tool.name().yellow(),
        version.yellow()
    );

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
    println!("{} from {}...", "Downloading".cyan(), download_url.dimmed());
    download_with_retry(&download_url, &archive_path, 3)?;

    // 2. 验证 checksum
    if let Ok(Some(expected)) = tool.get_checksum(version, arch) {
        println!("{}...", "Verifying checksum".cyan());
        verify_checksum(&archive_path, &expected)?;
        println!("{} Checksum verified", "✓".green());
    }

    // 3. 解压
    println!("{}...", "Extracting".cyan());
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
        "{} Installed {} {} to {}",
        "✓".green(),
        tool.name().yellow(),
        version.yellow(),
        final_dir.display().to_string().dimmed()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_check_disk_space_sufficient() {
        let temp_dir = TempDir::new().unwrap();
        // Request 1 byte - should always succeed
        let result = check_disk_space(temp_dir.path(), 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_disk_space_insufficient() {
        let temp_dir = TempDir::new().unwrap();
        // Request an impossibly large amount (1 PB)
        let result = check_disk_space(temp_dir.path(), 1024 * 1024 * 1024 * 1024 * 1024);
        assert!(result.is_err());
        if let Err(VexError::DiskSpace { need, available }) = result {
            assert!(need > available);
        } else {
            panic!("Expected DiskSpace error");
        }
    }

    #[test]
    fn test_min_free_space_constant() {
        // Verify the constant is set to 500 MB
        assert_eq!(MIN_FREE_SPACE_BYTES, 500 * 1024 * 1024);
    }
}
