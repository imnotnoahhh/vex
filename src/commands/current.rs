use crate::config;
use crate::error::{Result, VexError};
use crate::output::{print_json, OutputMode};
use crate::resolver;
use crate::ui;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct CurrentEntry {
    pub tool: String,
    pub version: String,
    pub source: String,
    pub source_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CurrentReport {
    pub cwd: String,
    pub tools: Vec<CurrentEntry>,
}

pub fn show(output: OutputMode) -> Result<()> {
    let report = collect_current()?;

    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render_text(&report);
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
    let global_versions = read_tool_versions(&global_path);

    let mut tools = Vec::new();

    for entry in fs::read_dir(&current_dir)?.filter_map(|e| e.ok()) {
        let tool_name = entry.file_name().to_string_lossy().to_string();
        let target = match fs::read_link(entry.path()) {
            Ok(target) => target,
            Err(_) => continue,
        };
        let Some(version) = target.file_name() else {
            continue;
        };
        let version_str = version.to_string_lossy().to_string();
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
        });
    }

    tools.sort_by(|a, b| a.tool.cmp(&b.tool));

    Ok(CurrentReport {
        cwd: pwd.display().to_string(),
        tools,
    })
}

fn render_text(report: &CurrentReport) {
    if report.tools.is_empty() {
        ui::dimmed("No tools activated yet.");
        println!();
        ui::dimmed("Use 'vex install <tool>' to install a tool.");
        return;
    }

    ui::header("Current active versions:");

    let mut table = ui::Table::new();
    for tool in &report.tools {
        let row = vec![
            tool.tool.yellow().to_string(),
            "→".to_string(),
            tool.version.cyan().to_string(),
            format!("({})", tool.source.dimmed()),
        ];
        table = table.row(row);

        if let Some(source_path) = &tool.source_path {
            table = table.row(vec![
                "".to_string(),
                "".to_string(),
                format!("{}: {}", "Source".dimmed(), source_path.dimmed()),
            ]);
        }
    }
    table.render();

    println!();
}

fn read_tool_versions(path: &Path) -> HashMap<String, String> {
    let Ok(content) = fs::read_to_string(path) else {
        return HashMap::new();
    };

    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }

            let mut parts = line.split_whitespace();
            let tool = parts.next()?;
            let version = parts.next()?;
            Some((tool.to_string(), version.to_string()))
        })
        .collect()
}

fn resolve_source(
    pwd: &Path,
    tool_name: &str,
    version_str: &str,
    versions: &HashMap<String, String>,
    global_path: &Path,
    global_versions: &HashMap<String, String>,
) -> (String, Option<String>) {
    let Some(project_version) = versions.get(tool_name) else {
        return global_or_manual(tool_name, version_str, global_path, global_versions);
    };

    if project_version != version_str {
        return global_or_manual(tool_name, version_str, global_path, global_versions);
    }

    find_project_source(pwd, tool_name)
        .map(|source_path| ("Project override".to_string(), Some(source_path)))
        .unwrap_or_else(|| global_or_manual(tool_name, version_str, global_path, global_versions))
}

fn global_or_manual(
    tool_name: &str,
    version_str: &str,
    global_path: &Path,
    global_versions: &HashMap<String, String>,
) -> (String, Option<String>) {
    match global_versions.get(tool_name) {
        Some(global_version) if global_version == version_str => (
            "Global default".to_string(),
            Some(global_path.display().to_string()),
        ),
        _ => ("Manual activation".to_string(), None),
    }
}

fn find_project_source(start_dir: &Path, tool_name: &str) -> Option<String> {
    let mut dir = start_dir.to_path_buf();

    loop {
        let tool_versions = dir.join(".tool-versions");
        if tool_versions.is_file() {
            let matches_tool = fs::read_to_string(&tool_versions)
                .ok()
                .map(|content| {
                    content.lines().any(|line| {
                        line.split_whitespace()
                            .next()
                            .map(|name| name == tool_name)
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if matches_tool {
                return Some(tool_versions.display().to_string());
            }
        }

        if let Some(path) = find_tool_specific_version_file(&dir, tool_name) {
            return Some(path.display().to_string());
        }

        if !dir.pop() {
            break;
        }
    }

    None
}

fn find_tool_specific_version_file(dir: &Path, tool_name: &str) -> Option<PathBuf> {
    let candidates = [
        (".node-version", "node"),
        (".nvmrc", "node"),
        (".go-version", "go"),
        (".java-version", "java"),
        (".rust-toolchain", "rust"),
        (".python-version", "python"),
    ];

    candidates
        .iter()
        .find(|(_, tool)| *tool == tool_name)
        .map(|(file, _)| dir.join(file))
        .filter(|path| path.is_file())
}
