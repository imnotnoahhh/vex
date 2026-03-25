//! Lifecycle and advisory information for toolchains
//!
//! Provides lifecycle status (EOL, LTS, security updates) for installed versions.
//! Used by `outdated`, `doctor`, `install`, and `use` commands to warn users about
//! problematic versions.

mod java;
mod node;
mod python;
#[cfg(test)]
mod tests;

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

    /// Check if this advisory should trigger a warning
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
        "node" => node::node_advisory(version),
        "java" => java::java_advisory(version),
        "python" => python::python_advisory(version),
        _ => Advisory::new(AdvisoryStatus::Unknown),
    }
}
