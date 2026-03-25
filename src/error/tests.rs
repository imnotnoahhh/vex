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
        suggestions: "\n\nDid you mean:\n  - 20.11.0 (latest in 20.x)".to_string(),
    };
    assert!(err.to_string().contains("Version not found: node@99.0.0"));
    assert!(err.to_string().contains("Did you mean"));
    assert!(err.to_string().contains("vex list-remote"));
}

#[test]
fn test_error_display_version_not_found_no_suggestions() {
    let err = VexError::VersionNotFound {
        tool: "node".to_string(),
        version: "99.0.0".to_string(),
        suggestions: String::new(),
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

#[test]
fn test_error_display_unsupported_architecture() {
    let err = VexError::UnsupportedArchitecture("sparc64".to_string());
    assert!(err
        .to_string()
        .contains("Unsupported architecture: sparc64"));
    assert!(err.to_string().contains("Apple Silicon"));
}
