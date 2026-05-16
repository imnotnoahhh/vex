mod collectors;
mod helpers;
mod render;

use crate::commands::current;
use crate::config;
use crate::error::{Result, VexError};
use crate::output::{print_json, OutputMode};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct GlobalCliEntry {
    pub tool: String,
    pub name: String,
    pub kind: String,
    pub path: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_source_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GlobalsReport {
    pub cwd: String,
    pub entries: Vec<GlobalCliEntry>,
}

#[derive(Debug, Clone)]
pub(super) struct VersionContext {
    pub version: String,
    pub source: String,
    pub source_path: Option<String>,
}

pub fn show(tool_filter: Option<&str>, output: OutputMode, verbose: bool) -> Result<()> {
    let report = collect(tool_filter)?;
    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render::render_text(&report, verbose);
            Ok(())
        }
    }
}

pub fn collect(tool_filter: Option<&str>) -> Result<GlobalsReport> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let cwd = std::env::current_dir()?;
    let contexts = current_contexts().unwrap_or_default();
    let mut entries = Vec::new();

    collectors::node(&vex_dir, &contexts, tool_filter, &mut entries);
    collectors::python(&vex_dir, &contexts, tool_filter, &mut entries)?;
    collectors::go(&vex_dir, &contexts, tool_filter, &mut entries);
    collectors::rust(&vex_dir, &contexts, tool_filter, &mut entries);
    collectors::java(&contexts, tool_filter, &mut entries);

    entries.sort_by(|left, right| {
        left.tool
            .cmp(&right.tool)
            .then(left.kind.cmp(&right.kind))
            .then(left.name.cmp(&right.name))
            .then(left.path.cmp(&right.path))
    });

    Ok(GlobalsReport {
        cwd: cwd.display().to_string(),
        entries,
    })
}

fn current_contexts() -> Result<BTreeMap<String, VersionContext>> {
    Ok(current::collect_current()?
        .tools
        .into_iter()
        .map(|entry| {
            (
                entry.tool,
                VersionContext {
                    version: entry.version,
                    source: entry.source,
                    source_path: entry.source_path,
                },
            )
        })
        .collect())
}

#[cfg(test)]
mod tests;
