//! Lifecycle and advisory information for toolchains
//!
//! Provides lifecycle status (EOL, LTS, security updates) for installed versions.
//! Used by `outdated`, `doctor`, `install`, and `use` commands to warn users about
//! problematic versions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Advisory status for a version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvisoryStatus {
    /// Version is current and recommended
    Current,
    /// Version is end-of-life
    Eol,
    /// Version is near end-of-life (within 3 months)
    NearEol,
    /// A newer LTS version is available
    LtsAvailable,
    /// Security or bugfix update available in same major version
    SecurityUpdateAvailable,
    /// No advisory information available
    Unknown,
}

/// Advisory information for a specific version
#[derive(Debug, Clone, Serialize)]
pub struct Advisory {
    pub status: AdvisoryStatus,
    pub message: Option<String>,
    pub recommendation: Option<String>,
    pub eol_date: Option<DateTime<Utc>>,
}

impl Advisory {
    /// Create a new advisory
    pub fn new(status: AdvisoryStatus) -> Self {
        Self {
            status,
            message: None,
            recommendation: None,
            eol_date: None,
        }
    }

    /// Set advisory message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Set recommendation
    pub fn with_recommendation(mut self, recommendation: String) -> Self {
        self.recommendation = Some(recommendation);
        self
    }

    /// Set EOL date
    #[allow(dead_code)]
    pub fn with_eol_date(mut self, eol_date: DateTime<Utc>) -> Self {
        self.eol_date = Some(eol_date);
        self
    }

    /// Check if this advisory should trigger a warning
    #[allow(dead_code)]
    pub fn is_warning(&self) -> bool {
        matches!(
            self.status,
            AdvisoryStatus::Eol
                | AdvisoryStatus::NearEol
                | AdvisoryStatus::LtsAvailable
                | AdvisoryStatus::SecurityUpdateAvailable
        )
    }
}

/// Get advisory for a specific tool and version
pub fn get_advisory(tool: &str, version: &str) -> Advisory {
    match tool {
        "node" => node_advisory(version),
        "java" => java_advisory(version),
        "python" => python_advisory(version),
        _ => Advisory::new(AdvisoryStatus::Unknown),
    }
}

/// Node.js lifecycle advisory
fn node_advisory(version: &str) -> Advisory {
    let version = version.trim_start_matches('v');
    let major = version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    // Node.js LTS schedule (as of 2026-03)
    match major {
        // EOL versions
        0..=15 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("node@{} is end-of-life", major))
            .with_recommendation("upgrade to node@22 (current LTS)".to_string()),
        16 => Advisory::new(AdvisoryStatus::Eol)
            .with_message("node@16 is end-of-life".to_string())
            .with_recommendation("upgrade to node@22 (current LTS)".to_string()),
        17 => Advisory::new(AdvisoryStatus::Eol)
            .with_message("node@17 is end-of-life".to_string())
            .with_recommendation("upgrade to node@22 (current LTS)".to_string()),
        19 => Advisory::new(AdvisoryStatus::Eol)
            .with_message("node@19 is end-of-life".to_string())
            .with_recommendation("upgrade to node@22 (current LTS)".to_string()),
        21 => Advisory::new(AdvisoryStatus::Eol)
            .with_message("node@21 is end-of-life".to_string())
            .with_recommendation("upgrade to node@22 (current LTS)".to_string()),
        // Current LTS versions
        18 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("node@18 is in maintenance mode".to_string())
            .with_recommendation("consider upgrading to node@22 (current LTS)".to_string()),
        20 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("node@20 is in maintenance mode".to_string())
            .with_recommendation("consider upgrading to node@22 (current LTS)".to_string()),
        22 => Advisory::new(AdvisoryStatus::Current),
        23 => Advisory::new(AdvisoryStatus::Current),
        // Future versions
        _ => Advisory::new(AdvisoryStatus::Current),
    }
}

/// Java lifecycle advisory
fn java_advisory(version: &str) -> Advisory {
    let version = version.trim_start_matches('v');
    let major = version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    // Java LTS versions: 8, 11, 17, 21
    match major {
        0..=7 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("java@{} is end-of-life", major))
            .with_recommendation("upgrade to java@21 (current LTS)".to_string()),
        8 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("java@8 is very old".to_string())
            .with_recommendation("upgrade to java@21 (current LTS)".to_string()),
        9 | 10 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("java@{} is end-of-life", major))
            .with_recommendation("upgrade to java@21 (current LTS)".to_string()),
        11 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("java@11 is an older LTS".to_string())
            .with_recommendation("consider upgrading to java@21 (current LTS)".to_string()),
        12..=16 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("java@{} is end-of-life", major))
            .with_recommendation("upgrade to java@21 (current LTS)".to_string()),
        17 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("java@17 is an older LTS".to_string())
            .with_recommendation("consider upgrading to java@21 (current LTS)".to_string()),
        18..=20 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("java@{} is end-of-life", major))
            .with_recommendation("upgrade to java@21 (current LTS)".to_string()),
        21 => Advisory::new(AdvisoryStatus::Current),
        22 => Advisory::new(AdvisoryStatus::Current),
        23 => Advisory::new(AdvisoryStatus::Current),
        _ => Advisory::new(AdvisoryStatus::Current),
    }
}

/// Python lifecycle advisory
fn python_advisory(version: &str) -> Advisory {
    let version = version.trim_start_matches('v');
    let parts: Vec<&str> = version.split('.').collect();
    let major = parts
        .first()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let minor = parts
        .get(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    // Python 2 is EOL
    if major == 2 {
        return Advisory::new(AdvisoryStatus::Eol)
            .with_message("python@2 is end-of-life".to_string())
            .with_recommendation("upgrade to python@3.12 or python@3.13".to_string());
    }

    // Python 3.x lifecycle
    match minor {
        0..=7 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("python@3.{} is end-of-life", minor))
            .with_recommendation("upgrade to python@3.12 or python@3.13".to_string()),
        8 => Advisory::new(AdvisoryStatus::NearEol)
            .with_message("python@3.8 is near end-of-life".to_string())
            .with_recommendation("upgrade to python@3.12 or python@3.13".to_string()),
        9 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("python@3.9 is in security-only mode".to_string())
            .with_recommendation("consider upgrading to python@3.12 or python@3.13".to_string()),
        10 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("python@3.10 is in security-only mode".to_string())
            .with_recommendation("consider upgrading to python@3.12 or python@3.13".to_string()),
        11 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("python@3.11 is stable".to_string())
            .with_recommendation("consider upgrading to python@3.12 or python@3.13".to_string()),
        12 => Advisory::new(AdvisoryStatus::Current),
        13 => Advisory::new(AdvisoryStatus::Current),
        _ => Advisory::new(AdvisoryStatus::Current),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_eol_versions() {
        let advisory = node_advisory("16.20.0");
        assert_eq!(advisory.status, AdvisoryStatus::Eol);
        assert!(advisory.is_warning());
        assert!(advisory.message.is_some());
        assert!(advisory.recommendation.is_some());
    }

    #[test]
    fn test_node_current_lts() {
        let advisory = node_advisory("22.0.0");
        assert_eq!(advisory.status, AdvisoryStatus::Current);
        assert!(!advisory.is_warning());
    }

    #[test]
    fn test_node_older_lts() {
        let advisory = node_advisory("18.20.0");
        assert_eq!(advisory.status, AdvisoryStatus::LtsAvailable);
        assert!(advisory.is_warning());
    }

    #[test]
    fn test_java_eol_versions() {
        let advisory = java_advisory("10.0.0");
        assert_eq!(advisory.status, AdvisoryStatus::Eol);
        assert!(advisory.is_warning());
    }

    #[test]
    fn test_java_current_lts() {
        let advisory = java_advisory("21.0.0");
        assert_eq!(advisory.status, AdvisoryStatus::Current);
        assert!(!advisory.is_warning());
    }

    #[test]
    fn test_java_older_lts() {
        let advisory = java_advisory("17.0.0");
        assert_eq!(advisory.status, AdvisoryStatus::LtsAvailable);
        assert!(advisory.is_warning());
    }

    #[test]
    fn test_python_eol_versions() {
        let advisory = python_advisory("3.7.0");
        assert_eq!(advisory.status, AdvisoryStatus::Eol);
        assert!(advisory.is_warning());
    }

    #[test]
    fn test_python_current() {
        let advisory = python_advisory("3.12.0");
        assert_eq!(advisory.status, AdvisoryStatus::Current);
        assert!(!advisory.is_warning());
    }

    #[test]
    fn test_python2_eol() {
        let advisory = python_advisory("2.7.18");
        assert_eq!(advisory.status, AdvisoryStatus::Eol);
        assert!(advisory.is_warning());
    }

    #[test]
    fn test_unsupported_tool() {
        let advisory = get_advisory("go", "1.21.0");
        assert_eq!(advisory.status, AdvisoryStatus::Unknown);
        assert!(!advisory.is_warning());
    }
}
