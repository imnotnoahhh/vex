//! 统一错误处理模块
//!
//! 定义 vex 中所有可能出现的错误类型 [`VexError`]，
//! 使用 [`thiserror`] 自动派生 `Display` 和 `Error`。
//! 每个变体都包含用户友好的故障排除建议。

use std::path::PathBuf;
use thiserror::Error;

/// vex 统一错误类型
///
/// 涵盖网络、IO、校验和、版本查找、锁冲突等所有错误场景。
/// 每个变体的 `Display` 输出都附带故障排除建议。
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum VexError {
    /// 网络请求失败（连接超时、DNS 解析失败等）
    #[error("Network error: {0}\n\nTroubleshooting:\n  - Check your internet connection\n  - Verify firewall settings\n  - Try again in a few moments")]
    Network(#[from] reqwest::Error),

    /// IO 操作失败（文件读写、权限不足等）
    #[error("IO error: {0}\n\nThis may be caused by:\n  - Insufficient permissions\n  - Disk full\n  - File system issues")]
    Io(#[from] std::io::Error),

    /// 磁盘空间不足（安装前检查，至少需要 500 MB）
    #[error("Disk space insufficient: need {need} GB, available {available} GB\n\nSuggestions:\n  - Free up disk space by removing unused files\n  - Run 'vex uninstall <tool@version>' to remove old versions\n  - Check disk usage with 'df -h'")]
    DiskSpace {
        /// 需要的空间（GB）
        need: u64,
        /// 可用空间（GB）
        available: u64,
    },

    /// 文件权限不足
    #[error("Permission denied: {path}\n\nTo fix this:\n  - Run with appropriate permissions\n  - Check file ownership: ls -la {path}\n  - You may need to run: chmod +x {path}")]
    Permission {
        /// 无权限访问的路径
        path: PathBuf,
    },

    /// SHA256 校验和不匹配，下载文件可能已损坏
    #[error("Checksum mismatch: expected {expected}, got {actual}\n\nThis indicates:\n  - Download was corrupted\n  - Network transmission error\n  - Potential security issue\n\nSuggestion: Try downloading again with 'vex install <tool@version>'")]
    ChecksumMismatch {
        /// 期望的校验和
        expected: String,
        /// 实际计算的校验和
        actual: String,
    },

    /// 指定的工具版本不存在或未安装
    #[error("Version not found: {tool}@{version}\n\nTo find available versions:\n  - Run 'vex list-remote {tool}' to see all versions\n  - Run 'vex alias {tool}' to see version aliases\n  - Check https://github.com/imnotnoahhh/vex for supported tools")]
    VersionNotFound {
        /// 工具名称
        tool: String,
        /// 版本号
        version: String,
    },

    /// 不支持的工具名称（当前支持 node、go、java、rust）
    #[error("Tool not found: {0}\n\nSupported tools: node, go, java, rust\n\nTo see available versions:\n  - Run 'vex list-remote <tool>'\n  - Visit https://github.com/imnotnoahhh/vex for documentation")]
    ToolNotFound(String),

    /// 解析错误（版本号格式、配置文件格式等）
    #[error("Parse error: {0}\n\nExpected format:\n  - tool@version (e.g., node@20.11.0)\n  - tool@alias (e.g., node@latest)\n  - tool (for interactive selection)")]
    Parse(String),

    /// 交互式对话框错误（非交互终端等）
    #[error("Dialog error: {0}\n\nThis may happen if:\n  - Terminal doesn't support interactive input\n  - Running in non-interactive mode\n\nTry: Specify version explicitly (e.g., 'vex install node@20')")]
    Dialog(String),

    /// 安装锁冲突，另一个 vex 进程正在安装同一版本
    #[error("Another vex process is installing {tool}@{version}\n\nPlease wait for the other installation to complete, then try again.\n\nIf you're sure no other process is running:\n  - Check for stale lock files in ~/.vex/locks/\n  - Remove lock file: rm ~/.vex/locks/{tool}-{version}.lock")]
    LockConflict {
        /// 工具名称
        tool: String,
        /// 版本号
        version: String,
    },

    /// 无法确定用户主目录（HOME 未设置）
    #[error("Could not determine home directory\n\nPlease ensure:\n  - HOME environment variable is set\n  - You have a valid home directory\n  - Check with: echo $HOME")]
    HomeDirectoryNotFound,
}

/// vex 的 Result 类型别名，等价于 `std::result::Result<T, VexError>`
pub type Result<T> = std::result::Result<T, VexError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_tool_not_found() {
        let err = VexError::ToolNotFound("python".to_string());
        assert!(err.to_string().contains("Tool not found: python"));
        assert!(err.to_string().contains("Supported tools"));
    }

    #[test]
    fn test_error_display_version_not_found() {
        let err = VexError::VersionNotFound {
            tool: "node".to_string(),
            version: "99.0.0".to_string(),
        };
        assert!(err.to_string().contains("Version not found: node@99.0.0"));
        assert!(err.to_string().contains("vex list-remote"));
    }

    #[test]
    fn test_error_display_parse() {
        let err = VexError::Parse("bad format".to_string());
        assert!(err.to_string().contains("Parse error: bad format"));
        assert!(err.to_string().contains("Expected format"));
    }

    #[test]
    fn test_error_display_dialog() {
        let err = VexError::Dialog("cancelled".to_string());
        assert!(err.to_string().contains("Dialog error: cancelled"));
        assert!(err.to_string().contains("non-interactive"));
    }

    #[test]
    fn test_error_display_checksum_mismatch() {
        let err = VexError::ChecksumMismatch {
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        assert!(err.to_string().contains("Checksum mismatch"));
        assert!(err.to_string().contains("corrupted"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let vex_err: VexError = io_err.into();
        assert!(matches!(vex_err, VexError::Io(_)));
        assert!(vex_err.to_string().contains("file missing"));
    }

    #[test]
    fn test_error_display_disk_space() {
        let err = VexError::DiskSpace {
            need: 5,
            available: 1,
        };
        assert!(err.to_string().contains("Disk space insufficient"));
        assert!(err.to_string().contains("5 GB"));
        assert!(err.to_string().contains("1 GB"));
    }

    #[test]
    fn test_error_display_permission() {
        let err = VexError::Permission {
            path: PathBuf::from("/usr/local/bin"),
        };
        assert!(err.to_string().contains("Permission denied"));
        assert!(err.to_string().contains("/usr/local/bin"));
    }

    #[test]
    fn test_error_display_lock_conflict() {
        let err = VexError::LockConflict {
            tool: "node".to_string(),
            version: "20.11.0".to_string(),
        };
        assert!(err.to_string().contains("Another vex process"));
        assert!(err.to_string().contains("node@20.11.0"));
    }

    #[test]
    fn test_error_display_home_directory_not_found() {
        let err = VexError::HomeDirectoryNotFound;
        assert!(err
            .to_string()
            .contains("Could not determine home directory"));
        assert!(err.to_string().contains("HOME environment variable"));
    }
}
