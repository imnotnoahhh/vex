mod render;
mod source;

use crate::config;
use crate::error::{Result, VexError};
use crate::output::{print_json, OutputMode};
use crate::resolver;
use crate::tool_metadata::{self, ToolchainMetadata};
use render::render_text;
use serde::Serialize;
use source::resolve_source;
use std::fs;

#[derive(Debug, Serialize)]
pub struct CurrentEntry {
    pub tool: String,
    pub version: String,
    pub source: String,
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ToolchainMetadata>,
}

#[derive(Debug, Serialize)]
pub struct CurrentReport {
    pub cwd: String,
    pub tools: Vec<CurrentEntry>,
}

pub fn show(output: OutputMode, verbose: bool) -> Result<()> {
    let report = collect_current()?;

    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render_text(&report, verbose);
            Ok(())
        }
    }
}

pub fn collect_current() -> Result<CurrentReport> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let current_dir = vex_dir.join("current");
    let pwd = resolver::current_dir();
    let _ = config::load_effective_settings(&pwd)?;

    if !current_dir.exists() {
        return Ok(CurrentReport {
            cwd: pwd.display().to_string(),
            tools: Vec::new(),
        });
    }

    let versions = resolver::resolve_versions(&pwd);
    let global_path = vex_dir.join("tool-versions");
    let global_versions = resolver::read_tool_versions_file(&global_path);

    let mut tools = Vec::new();

    for entry in fs::read_dir(&current_dir)?.filter_map(|entry| entry.ok()) {
        let tool_name = entry.file_name().to_string_lossy().to_string();
        let target = match fs::read_link(entry.path()) {
            Ok(target) => target,
            Err(_) => continue,
        };
        let Some(version) = target.file_name() else {
            continue;
        };
        let version_str = version.to_string_lossy().to_string();
        let install_dir = if target.is_absolute() {
            target.clone()
        } else {
            current_dir.join(&target)
        };
        let (source, source_path) = resolve_source(
            &pwd,
            &tool_name,
            &version_str,
            &versions,
            &global_path,
            &global_versions,
        );

        tools.push(CurrentEntry {
            tool: tool_name,
            version: version_str,
            source,
            source_path,
            metadata: tool_metadata::read_metadata(&install_dir)?,
        });
    }

    tools.sort_by(|left, right| left.tool.cmp(&right.tool));

    Ok(CurrentReport {
        cwd: pwd.display().to_string(),
        tools,
    })
}
