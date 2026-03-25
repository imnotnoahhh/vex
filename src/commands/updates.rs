mod outdated;
mod render;
mod targets;
mod upgrade;

use crate::error::{Result, VexError};
use crate::output::{print_json, OutputMode};
use serde::Serialize;

use render::{render_outdated_text, render_upgrade_text};
use upgrade::{upgrade_all, upgrade_one};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedSource {
    Project,
    Global,
    Active,
    Installed,
}

#[derive(Debug, Serialize)]
pub struct OutdatedEntry {
    pub tool: String,
    pub current_version: String,
    pub latest_version: String,
    pub status: String,
    pub source: ManagedSource,
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_recommendation: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OutdatedReport {
    pub scope: String,
    pub entries: Vec<OutdatedEntry>,
}

#[derive(Debug, Serialize)]
pub struct UpgradeEntry {
    pub tool: String,
    pub previous_version: String,
    pub target_version: String,
    pub status: String,
    pub source: ManagedSource,
    pub source_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpgradeReport {
    pub scope: String,
    pub entries: Vec<UpgradeEntry>,
}

pub fn outdated(tool: Option<&str>, output: OutputMode) -> Result<()> {
    let report = outdated::collect_outdated(tool)?;
    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render_outdated_text(&report);
            Ok(())
        }
    }
}

pub fn upgrade(tool: Option<&str>, all: bool) -> Result<()> {
    let report = if all {
        upgrade_all()?
    } else if let Some(tool) = tool {
        upgrade_one(tool)?
    } else {
        return Err(VexError::Parse(
            "Please specify a tool (e.g., 'vex upgrade node') or use --all".to_string(),
        ));
    };

    render_upgrade_text(&report);
    Ok(())
}
