use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum VexError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Disk space insufficient: need {need} GB, available {available} GB")]
    DiskSpace { need: u64, available: u64 },

    #[error("Permission denied: {path}")]
    Permission { path: PathBuf },

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Version not found: {tool}@{version}. Run 'vex list-remote {tool}' to see available versions")]
    VersionNotFound { tool: String, version: String },

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Dialog error: {0}")]
    Dialog(String),
}

pub type Result<T> = std::result::Result<T, VexError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_tool_not_found() {
        let err = VexError::ToolNotFound("python".to_string());
        assert_eq!(err.to_string(), "Tool not found: python");
    }

    #[test]
    fn test_error_display_version_not_found() {
        let err = VexError::VersionNotFound {
            tool: "node".to_string(),
            version: "99.0.0".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Version not found: node@99.0.0. Run 'vex list-remote node' to see available versions"
        );
    }

    #[test]
    fn test_error_display_parse() {
        let err = VexError::Parse("bad format".to_string());
        assert_eq!(err.to_string(), "Parse error: bad format");
    }

    #[test]
    fn test_error_display_dialog() {
        let err = VexError::Dialog("cancelled".to_string());
        assert_eq!(err.to_string(), "Dialog error: cancelled");
    }

    #[test]
    fn test_error_display_checksum_mismatch() {
        let err = VexError::ChecksumMismatch {
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        assert_eq!(err.to_string(), "Checksum mismatch: expected abc, got def");
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
        assert_eq!(
            err.to_string(),
            "Disk space insufficient: need 5 GB, available 1 GB"
        );
    }

    #[test]
    fn test_error_display_permission() {
        let err = VexError::Permission {
            path: PathBuf::from("/usr/local/bin"),
        };
        assert_eq!(err.to_string(), "Permission denied: /usr/local/bin");
    }
}
