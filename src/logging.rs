//! Unified logging framework for vex
//!
//! This module provides structured logging using the `tracing` crate.
//! Logging is opt-in via `VEX_LOG` so the normal CLI UX stays quiet unless
//! the user explicitly requests diagnostic output.

use tracing_subscriber::{fmt, EnvFilter};

const LOG_ENV: &str = "VEX_LOG";

fn env_filter() -> EnvFilter {
    EnvFilter::try_from_env(LOG_ENV)
        .or_else(|_| EnvFilter::try_new("off"))
        .unwrap()
}

pub fn diagnostics_enabled() -> bool {
    matches!(
        std::env::var(LOG_ENV),
        Ok(value) if !value.trim().is_empty() && !value.eq_ignore_ascii_case("off")
    )
}

/// Initialize the logging system
///
/// This should be called once at the start of the application.
/// Log level can be controlled via the `VEX_LOG` environment variable:
/// - `VEX_LOG=trace` - Most verbose
/// - `VEX_LOG=debug` - Debug information
/// - `VEX_LOG=info` - General information
/// - `VEX_LOG=warn` - Warnings only
/// - `VEX_LOG=error` - Errors only
/// - unset - logging disabled (default)
///
/// # Example
/// ```no_run
/// vex::logging::init();
/// ```
pub fn init() {
    fmt()
        .with_env_filter(env_filter())
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn test_logging_init() {
        let _guard = env_lock().lock().unwrap();
        std::env::remove_var(LOG_ENV);
        // Test that init doesn't panic
        // Note: Can only be called once per process
        init();
    }

    #[test]
    fn test_env_filter_defaults_to_off() {
        let _guard = env_lock().lock().unwrap();
        std::env::remove_var(LOG_ENV);
        assert_eq!(env_filter().to_string(), "off");
    }

    #[test]
    fn test_custom_log_level() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var(LOG_ENV, "debug");
        assert_eq!(env_filter().to_string(), "debug");
        assert!(diagnostics_enabled());
        std::env::remove_var(LOG_ENV);
    }

    #[test]
    fn test_off_disables_diagnostics_mode() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var(LOG_ENV, "off");
        assert!(!diagnostics_enabled());
        std::env::remove_var(LOG_ENV);
    }
}
