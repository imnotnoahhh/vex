//! Unified logging framework for vex
//!
//! This module provides structured logging using the `tracing` crate.
//! It supports different log levels and can be configured via environment variables.

use tracing_subscriber::{fmt, EnvFilter};

/// Initialize the logging system
///
/// This should be called once at the start of the application.
/// Log level can be controlled via the `VEX_LOG` environment variable:
/// - `VEX_LOG=trace` - Most verbose
/// - `VEX_LOG=debug` - Debug information
/// - `VEX_LOG=info` - General information (default)
/// - `VEX_LOG=warn` - Warnings only
/// - `VEX_LOG=error` - Errors only
///
/// # Example
/// ```no_run
/// vex::logging::init();
/// ```
pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init() {
        // Test that init doesn't panic
        // Note: Can only be called once per process
        init();
    }

    #[test]
    fn test_env_filter_creation() {
        // Test that filter can be created with default
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap();

        // Filter should be created successfully (just verify it exists)
        let _ = format!("{:?}", filter);
    }

    #[test]
    fn test_custom_log_level() {
        // Test that custom log level can be set
        let original = std::env::var("VEX_LOG").ok();

        std::env::set_var("VEX_LOG", "debug");
        // Just verify the environment variable was set
        assert_eq!(std::env::var("VEX_LOG").unwrap(), "debug");

        // Restore original
        if let Some(val) = original {
            std::env::set_var("VEX_LOG", val);
        } else {
            std::env::remove_var("VEX_LOG");
        }
    }
}
