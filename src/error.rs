//! Unified error handling module
//!
//! Defines all possible error types in vex [`VexError`],
//! using [`thiserror`] to automatically derive `Display` and `Error`.
//! Each variant includes user-friendly troubleshooting suggestions.

use thiserror::Error;

/// vex unified error type
///
/// Covers all error scenarios including network, IO, checksum, version lookup, lock conflicts, etc.
/// Each variant's `Display` output includes troubleshooting suggestions.
#[derive(Error, Debug)]
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

    /// SHA256 checksum mismatch, downloaded file may be corrupted
    #[error("Checksum mismatch: expected {expected}, got {actual}\n\nThis indicates:\n  - Download was corrupted\n  - Network transmission error\n  - Potential security issue\n\nSuggestion: Try downloading again with 'vex install <tool@version>'")]
    ChecksumMismatch {
        /// Expected checksum
        expected: String,
        /// Actual calculated checksum
        actual: String,
    },

    /// Specified tool version does not exist or is not installed
    #[error("Version not found: {tool}@{version}{suggestions}\n\nRun 'vex list-remote {tool}' to see all available versions.")]
    VersionNotFound {
        /// Tool name
        tool: String,
        /// Version number
        version: String,
        /// Suggested versions
        suggestions: String,
    },

    /// Unsupported tool name (currently supports node, go, java, rust, python)
    #[error("Tool not found: {0}\n\nSupported tools: node, go, java, rust, python\n\nTo see available versions:\n  - Run 'vex list-remote <tool>'\n  - Visit https://github.com/imnotnoahhh/vex for documentation")]
    ToolNotFound(String),

    /// Parse error (version number format, configuration file format, etc.)
    #[error("Parse error: {0}\n\nExpected format:\n  - tool@version (e.g., node@20.11.0)\n  - tool@alias (e.g., node@latest)\n  - tool (tool name only)")]
    Parse(String),

    /// Configuration error (invalid config file, missing fields, lockfile mismatch, etc.)
    #[error("Configuration error: {0}")]
    Config(String),

    /// Python environment error (missing .venv, requirements.lock, etc.)
    #[error("{0}")]
    PythonEnv(String),

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

    /// Current CPU architecture is not supported by vex's binary adapters
    #[error("Unsupported architecture: {0}\n\nvex currently supports macOS on Apple Silicon (arm64) and Intel (x86_64).")]
    UnsupportedArchitecture(String),

    /// Offline mode error - required data not available in cache
    #[error("Offline mode error: {0}\n\nIn offline mode, vex can only use cached data.\n\nTo fix this:\n  - Run the command without --offline to fetch fresh data\n  - Ensure you have previously fetched the data while online\n  - Check cache directory: ~/.vex/cache/")]
    OfflineModeError(String),
}

/// vex's Result type alias, equivalent to `std::result::Result<T, VexError>`
pub type Result<T> = std::result::Result<T, VexError>;

#[cfg(test)]
mod tests;
