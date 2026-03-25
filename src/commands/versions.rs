mod collect;
mod filter;
mod render;

use crate::error::Result;
use crate::output::{print_json, OutputMode};
use serde::Serialize;

use collect::{collect_installed, collect_remote};
use render::{render_installed_text, render_remote_text};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum RemoteFilter {
    All,
    Lts,
    Major,
    Latest,
}

#[derive(Debug, Serialize)]
pub struct InstalledVersionEntry {
    pub version: String,
    pub is_current: bool,
}

#[derive(Debug, Serialize)]
pub struct InstalledVersionsReport {
    pub tool: String,
    pub current_version: Option<String>,
    pub versions: Vec<InstalledVersionEntry>,
}

#[derive(Debug, Serialize)]
pub struct RemoteVersionEntry {
    pub version: String,
    pub label: Option<String>,
    pub is_current: bool,
    pub is_outdated: bool,
}

#[derive(Debug, Serialize)]
pub struct RemoteVersionsReport {
    pub tool: String,
    pub filter: String,
    pub total: usize,
    pub current_version: Option<String>,
    pub versions: Vec<RemoteVersionEntry>,
}

pub fn list_installed(tool_name: &str, output: OutputMode) -> Result<()> {
    let report = collect_installed(tool_name)?;
    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render_installed_text(&report);
            Ok(())
        }
    }
}

pub fn list_remote(
    tool_name: &str,
    filter: RemoteFilter,
    use_cache: bool,
    offline: bool,
    output: OutputMode,
) -> Result<()> {
    let report = collect_remote(
        tool_name,
        filter,
        use_cache,
        offline,
        output == OutputMode::Text,
    )?;
    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render_remote_text(&report);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;
