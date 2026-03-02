//! Unified error handling module
//!
//! Defines all possible error types in vex [`VexError`],
//! using [`thiserror`] to automatically derive `Display` and `Error`.
//! Each variant includes user-friendly troubleshooting suggestions.

use std::path::PathBuf;
use thiserror::Error;

/// vex unified error type
///
/// Covers all error scenarios including network, IO, checksum, version lookup, lock conflicts, etc.
/// Each variant's `Display` output includes troubleshooting suggestions.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum VexError {
    /// Network request failed (connection timeout, DNS resolution failure, etc.)
    #[error("Network error: {0}\n\nTroubleshooting:\n  - Check your internet connection\n  - Verify firewall settings\n  - Try again in a few moments")]
    Network(#[from] reqwest::Error),

    /// IO operation failed (file read/write, insufficient permissions, etc.)
    #[error("IO error: {0}\n\nThis may be caused by:\n  - Insufficient permissions\n  - Disk full\n  - File system issues")]
    Io(#[from] std::io::Error),

    /// Insufficient disk space (pre-installation check, requires at least 500 MB)
    #[error("Disk space insufficient: need {need} GB, available {available} GB\n\nSuggestions:\n  - Free up disk space by removing unused files\n  - Run 'vex uninstall <tool@version>' to remove old versions\n  - Check disk usage with 'df -h'")]
    DiskSpace {
        /// Required space (GB)
        need: u64,
        /// Available space (GB)
        available: u64,
    },

    /// Insufficient file permissions
    #[error("Permission denied: {path}\n\nTo fix this:\n  - Run with appropriate permissions\n  - Check file ownership: ls -la {path}\n  - You may need to run: chmod +x {path}")]
    Permission {
        /// Path with insufficient permissions
        path: PathBuf,
    },

    /// SHA256 checksum mismatch, downloaded file may be corrupted
    #[error("Checksum mismatch: expected {expected}, got {actual}\n\nThis indicates:\n  - Download was corrupted\n  - Network transmission error\n  - Potential security issue\n\nSuggestion: Try downloading again with 'vex install <tool@version>'")]
    ChecksumMismatch {
        /// Expected checksum
        expected: String,
        /// Actual calculated checksum
        actual: String,
    },

    /// Specified tool version does not exist or is not installed
    #[error("Version not found: {tool}@{version}\n\nTo find available versions:\n  - Run 'vex list-remote {tool}' to see all versions\n  - Run 'vex alias {tool}' to see version aliases\n  - Check https://github.com/imnotnoahhh/vex for supported tools")]
    VersionNotFound {
        /// Tool name
        tool: String,
        /// Version number
        version: String,
    },

    /// Unsupported tool name (currently supports node, go, java, rust)
    #[error("Tool not found: {0}\n\nSupported tools: node, go, java, rust\n\nTo see available versions:\n  - Run 'vex list-remote <tool>'\n  - Visit https://github.com/imnotnoahhh/vex for documentation")]
    ToolNotFound(String),

    /// Parse error (version number format, configuration file format, etc.)
    #[error("Parse error: {0}\n\nExpected format:\n  - tool@version (e.g., node@20.11.0)\n  - tool@alias (e.g., node@latest)\n  - tool (for interactive selection)")]
    Parse(String),

    /// Interactive dialog error (non-interactive terminal, etc.)
    #[error("Dialog error: {0}\n\nThis may happen if:\n  - Terminal doesn't support interactive input\n  - Running in non-interactive mode\n\nTry: Specify version explicitly (e.g., 'vex install node@20')")]
    Dialog(String),

    /// Install lock conflict, another vex process is installing the same version
    #[error("Another vex process is installing {tool}@{version}\n\nPlease wait for the other installation to complete, then try again.\n\nIf you're sure no other process is running:\n  - Check for stale lock files in ~/.vex/locks/\n  - Remove lock file: rm ~/.vex/locks/{tool}-{version}.lock")]
    LockConflict {
        /// Tool name
        tool: String,
        /// Version number
        version: String,
    },

    /// Cannot determine user home directory (HOME not set)
    #[error("Could not determine home directory\n\nPlease ensure:\n  - HOME environment variable is set\n  - You have a valid home directory\n  - Check with: echo $HOME")]
    HomeDirectoryNotFound,
}

/// vex's Result type alias, equivalent to `std::result::Result<T, VexError>`
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
