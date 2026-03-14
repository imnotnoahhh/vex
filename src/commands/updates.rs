use crate::config;
use crate::error::{Result, VexError};
use crate::installer;
use crate::output::{print_json, OutputMode};
use crate::resolver;
use crate::switcher;
use crate::tools;
use crate::ui;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone)]
struct ManagedTarget {
    tool: String,
    version: String,
    source: ManagedSource,
    source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VersionFileFormat {
    ToolVersions,
    SingleValue,
}

pub fn outdated(tool: Option<&str>, output: OutputMode) -> Result<()> {
    let report = collect_outdated(tool)?;
    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render_outdated_text(&report);
            Ok(())
        }
    }
}

pub fn collect_outdated(tool: Option<&str>) -> Result<OutdatedReport> {
    let (scope, targets) = collect_targets(tool)?;
    let mut entries = Vec::new();

    for target in targets {
        let tool_impl = tools::get_tool(&target.tool)?;
        let latest_version = tools::resolve_fuzzy_version(tool_impl.as_ref(), "latest")?;
        let status = if normalize_version(&target.version) == normalize_version(&latest_version) {
            "up_to_date"
        } else {
            "outdated"
        };

        entries.push(OutdatedEntry {
            tool: target.tool,
            current_version: normalize_version(&target.version),
            latest_version: normalize_version(&latest_version),
            status: status.to_string(),
            source: target.source,
            source_path: target.source_path.map(|path| path.display().to_string()),
        });
    }

    entries.sort_by(|a, b| a.tool.cmp(&b.tool));
    Ok(OutdatedReport { scope, entries })
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

fn upgrade_all() -> Result<UpgradeReport> {
    let (scope, targets) = collect_targets(None)?;
    if targets.is_empty() {
        return Ok(UpgradeReport {
            scope,
            entries: Vec::new(),
        });
    }

    let mut entries = Vec::new();
    for target in targets {
        entries.push(upgrade_target(&target)?);
    }

    entries.sort_by(|a, b| a.tool.cmp(&b.tool));
    Ok(UpgradeReport { scope, entries })
}

fn upgrade_one(tool_name: &str) -> Result<UpgradeReport> {
    let (scope, targets) = collect_targets(Some(tool_name))?;
    let target = targets
        .into_iter()
        .next()
        .ok_or_else(|| VexError::VersionNotFound {
            tool: tool_name.to_string(),
            version: "managed version".to_string(),
        })?;

    Ok(UpgradeReport {
        scope,
        entries: vec![upgrade_target(&target)?],
    })
}

fn upgrade_target(target: &ManagedTarget) -> Result<UpgradeEntry> {
    let tool = tools::get_tool(&target.tool)?;
    let latest = tools::resolve_fuzzy_version(tool.as_ref(), "latest")?;
    let previous_version = normalize_version(&target.version);
    let target_version = normalize_version(&latest);

    let status = if previous_version == target_version {
        "already_latest".to_string()
    } else {
        let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
        let install_dir = vex_dir
            .join("toolchains")
            .join(&target.tool)
            .join(&target_version);
        if !install_dir.exists() {
            installer::install(tool.as_ref(), &target_version)?;
        }
        switcher::switch_version(tool.as_ref(), &target_version)?;

        if matches!(
            target.source,
            ManagedSource::Project | ManagedSource::Global
        ) {
            if let Some(source_path) = &target.source_path {
                write_tool_version(source_path, &target.tool, &target_version)?;
            }
        }

        "upgraded".to_string()
    };

    Ok(UpgradeEntry {
        tool: target.tool.clone(),
        previous_version,
        target_version,
        status,
        source: target.source,
        source_path: target
            .source_path
            .as_ref()
            .map(|path| path.display().to_string()),
    })
}

fn render_outdated_text(report: &OutdatedReport) {
    if report.entries.is_empty() {
        ui::dimmed("No managed tools found in the current context.");
        return;
    }

    ui::header(&format!("Outdated check scope: {}", report.scope.cyan()));

    let mut table = ui::Table::new();
    let mut outdated_count = 0;

    for entry in &report.entries {
        let status = if entry.status == "outdated" {
            outdated_count += 1;
            "outdated".yellow().to_string()
        } else {
            "up to date".green().to_string()
        };

        table = table.row(vec![
            entry.tool.yellow().to_string(),
            entry.current_version.dimmed().to_string(),
            "→".to_string(),
            entry.latest_version.cyan().to_string(),
            format!("({})", status),
        ]);

        if let Some(path) = &entry.source_path {
            table = table.row(vec![
                "".to_string(),
                format!(
                    "{}: {} ({})",
                    "Source".dimmed(),
                    path.dimmed(),
                    source_label(entry.source).dimmed()
                ),
            ]);
        } else {
            table = table.row(vec![
                "".to_string(),
                format!(
                    "{}: {}",
                    "Source".dimmed(),
                    source_label(entry.source).dimmed()
                ),
            ]);
        }
    }

    table.render();

    println!();
    if outdated_count == 0 {
        ui::success("All managed tools are up to date.");
    } else {
        ui::info(&format!(
            "{} tool(s) are behind the latest available version",
            outdated_count
        ));
    }
}

fn render_upgrade_text(report: &UpgradeReport) {
    if report.entries.is_empty() {
        ui::dimmed("No managed tools found to upgrade.");
        return;
    }

    ui::header(&format!("Upgrade scope: {}", report.scope.cyan()));

    let mut summary = ui::Summary::new();

    for entry in &report.entries {
        let message = format!(
            "{}  {} → {}",
            entry.tool.yellow(),
            entry.previous_version.dimmed(),
            entry.target_version.cyan()
        );

        match entry.status.as_str() {
            "already_latest" => summary = summary.info(message),
            "upgraded" => {
                summary = summary.success(message);
                if let Some(path) = &entry.source_path {
                    summary = summary.info(format!(
                        "  Updated: {} ({})",
                        path.dimmed(),
                        source_label(entry.source).dimmed()
                    ));
                }
            }
            _ => summary = summary.info(message),
        }
    }

    summary.render();
}

fn collect_targets(tool_filter: Option<&str>) -> Result<(String, Vec<ManagedTarget>)> {
    let cwd = resolver::current_dir();
    let project_versions = resolve_project_versions(&cwd);
    let global_path = config::vex_home()
        .ok_or(VexError::HomeDirectoryNotFound)?
        .join("tool-versions");
    let global_versions = read_tool_versions(&global_path);
    let current_versions = read_current_versions()?;

    if let Some(tool_name) = tool_filter {
        let _ = tools::get_tool(tool_name)?;

        if let Some(version) = project_versions.get(tool_name) {
            return Ok((
                "explicit".to_string(),
                vec![ManagedTarget {
                    tool: tool_name.to_string(),
                    version: version.clone(),
                    source: ManagedSource::Project,
                    source_path: find_project_source(&cwd, tool_name),
                }],
            ));
        }

        if let Some(version) = global_versions.get(tool_name) {
            return Ok((
                "explicit".to_string(),
                vec![ManagedTarget {
                    tool: tool_name.to_string(),
                    version: version.clone(),
                    source: ManagedSource::Global,
                    source_path: Some(global_path),
                }],
            ));
        }

        if let Some(version) = current_versions.get(tool_name) {
            return Ok((
                "explicit".to_string(),
                vec![ManagedTarget {
                    tool: tool_name.to_string(),
                    version: version.clone(),
                    source: ManagedSource::Active,
                    source_path: None,
                }],
            ));
        }

        if let Some(version) = latest_installed_version(tool_name)? {
            return Ok((
                "explicit".to_string(),
                vec![ManagedTarget {
                    tool: tool_name.to_string(),
                    version,
                    source: ManagedSource::Installed,
                    source_path: None,
                }],
            ));
        }

        return Ok(("explicit".to_string(), Vec::new()));
    }

    if !project_versions.is_empty() {
        let mut targets = project_versions
            .iter()
            .map(|(tool, version)| ManagedTarget {
                tool: tool.clone(),
                version: version.clone(),
                source: ManagedSource::Project,
                source_path: find_project_source(&cwd, tool),
            })
            .collect::<Vec<_>>();
        targets.sort_by(|a, b| a.tool.cmp(&b.tool));
        return Ok(("project".to_string(), targets));
    }

    if !global_versions.is_empty() {
        let mut targets = global_versions
            .iter()
            .map(|(tool, version)| ManagedTarget {
                tool: tool.clone(),
                version: version.clone(),
                source: ManagedSource::Global,
                source_path: Some(global_path.clone()),
            })
            .collect::<Vec<_>>();
        targets.sort_by(|a, b| a.tool.cmp(&b.tool));
        return Ok(("global".to_string(), targets));
    }

    let mut targets = current_versions
        .into_iter()
        .map(|(tool, version)| ManagedTarget {
            tool,
            version,
            source: ManagedSource::Active,
            source_path: None,
        })
        .collect::<Vec<_>>();
    targets.sort_by(|a, b| a.tool.cmp(&b.tool));
    Ok(("active".to_string(), targets))
}

fn resolve_project_versions(start_dir: &Path) -> HashMap<String, String> {
    let mut versions = HashMap::new();
    let mut dir = start_dir.to_path_buf();

    loop {
        let tool_versions = dir.join(".tool-versions");
        if tool_versions.is_file() {
            for (tool, version) in read_tool_versions(&tool_versions) {
                versions.entry(tool).or_insert(version);
            }
        }

        for (file, tool) in [
            (".node-version", "node"),
            (".nvmrc", "node"),
            (".go-version", "go"),
            (".java-version", "java"),
            (".rust-toolchain", "rust"),
            (".python-version", "python"),
        ] {
            let path = dir.join(file);
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    let version = content.trim().to_string();
                    if !version.is_empty() {
                        versions.entry(tool.to_string()).or_insert(version);
                    }
                }
            }
        }

        if !dir.pop() {
            break;
        }
    }

    versions
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

fn read_current_versions() -> Result<HashMap<String, String>> {
    let current_dir = config::vex_home()
        .ok_or(VexError::HomeDirectoryNotFound)?
        .join("current");
    let mut versions = HashMap::new();

    if !current_dir.exists() {
        return Ok(versions);
    }

    for entry in fs::read_dir(&current_dir)?.filter_map(|e| e.ok()) {
        let tool = entry.file_name().to_string_lossy().to_string();
        let target = match fs::read_link(entry.path()) {
            Ok(target) => target,
            Err(_) => continue,
        };
        if let Some(version) = target.file_name() {
            versions.insert(tool, version.to_string_lossy().to_string());
        }
    }

    Ok(versions)
}

fn latest_installed_version(tool_name: &str) -> Result<Option<String>> {
    let tool_dir = config::vex_home()
        .ok_or(VexError::HomeDirectoryNotFound)?
        .join("toolchains")
        .join(tool_name);

    if !tool_dir.exists() {
        return Ok(None);
    }

    let mut versions = fs::read_dir(&tool_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().ok().map(|ft| ft.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    versions.sort_by_key(|version| version_sort_key(version));
    Ok(versions.pop())
}

fn find_project_source(start_dir: &Path, tool_name: &str) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    let mut seen = HashSet::new();

    loop {
        if !seen.insert(dir.clone()) {
            break;
        }

        let tool_versions = dir.join(".tool-versions");
        if tool_versions.is_file() {
            let contains_tool = read_tool_versions(&tool_versions).contains_key(tool_name);
            if contains_tool {
                return Some(tool_versions);
            }
        }

        for (file, tool) in [
            (".node-version", "node"),
            (".nvmrc", "node"),
            (".go-version", "go"),
            (".java-version", "java"),
            (".rust-toolchain", "rust"),
            (".python-version", "python"),
        ] {
            if tool == tool_name {
                let path = dir.join(file);
                if path.is_file() {
                    return Some(path);
                }
            }
        }

        if !dir.pop() {
            break;
        }
    }

    None
}

fn write_tool_version(file_path: &Path, tool_name: &str, version: &str) -> Result<()> {
    let content = match version_file_format(file_path) {
        VersionFileFormat::ToolVersions => {
            let mut entries: Vec<(String, String)> =
                read_tool_versions(file_path).into_iter().collect();
            entries.retain(|(tool, _)| tool != tool_name);
            entries.push((tool_name.to_string(), version.to_string()));
            entries.sort_by(|a, b| a.0.cmp(&b.0));

            entries
                .iter()
                .map(|(tool, version)| format!("{} {}", tool, version))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        }
        VersionFileFormat::SingleValue => format!("{}\n", version),
    };

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file_path, content)?;
    Ok(())
}

fn version_file_format(file_path: &Path) -> VersionFileFormat {
    match file_path.file_name().and_then(|name| name.to_str()) {
        Some(".tool-versions" | "tool-versions") => VersionFileFormat::ToolVersions,
        _ => VersionFileFormat::SingleValue,
    }
}

fn normalize_version(version: &str) -> String {
    version.strip_prefix('v').unwrap_or(version).to_string()
}

fn version_sort_key(version: &str) -> Vec<u32> {
    version
        .trim_start_matches('v')
        .split('.')
        .filter_map(|segment| segment.parse::<u32>().ok())
        .collect()
}

fn source_label(source: ManagedSource) -> &'static str {
    match source {
        ManagedSource::Project => "project",
        ManagedSource::Global => "global",
        ManagedSource::Active => "active",
        ManagedSource::Installed => "installed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_tool_version_preserves_tool_versions_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".tool-versions");
        fs::write(&path, "node 20.0.0\npython 3.12.0\n").unwrap();

        write_tool_version(&path, "node", "22.0.0").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "node 22.0.0\npython 3.12.0\n");
    }

    #[test]
    fn test_write_tool_version_preserves_global_tool_versions_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tool-versions");
        fs::write(&path, "python 3.13.12\n").unwrap();

        write_tool_version(&path, "python", "3.14.3").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "python 3.14.3\n");
    }

    #[test]
    fn test_write_tool_version_preserves_single_value_file_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".nvmrc");
        fs::write(&path, "20.0.0\n").unwrap();

        write_tool_version(&path, "node", "22.0.0").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "22.0.0\n");
    }
}
