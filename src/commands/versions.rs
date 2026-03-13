use crate::cache;
use crate::config;
use crate::error::{Result, VexError};
use crate::output::{print_json, OutputMode};
use crate::tools::{self, Tool, Version};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Copy)]
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
    output: OutputMode,
) -> Result<()> {
    let report = collect_remote(tool_name, filter, use_cache, output == OutputMode::Text)?;
    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render_remote_text(&report);
            Ok(())
        }
    }
}

pub fn fetch_versions_cached(tool: &dyn Tool, use_cache: bool) -> Result<Vec<Version>> {
    let vex = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let remote_cache = cache::RemoteCache::new(&vex);
    let ttl = config::cache_ttl()?.as_secs();

    if use_cache {
        if let Some(cached) = remote_cache.get_cached_versions(tool.name(), ttl) {
            return Ok(cached);
        }
    }

    let versions = tool.list_remote()?;
    remote_cache.set_cached_versions(tool.name(), &versions);
    Ok(versions)
}

pub fn get_current_version_for_tool(tool_name: &str) -> Option<String> {
    let current_link = config::current_dir()?.join(tool_name);

    fs::read_link(&current_link).ok().and_then(|target| {
        target
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
    })
}

pub fn collect_installed(tool_name: &str) -> Result<InstalledVersionsReport> {
    let toolchains_dir = config::toolchains_dir()
        .ok_or(VexError::HomeDirectoryNotFound)?
        .join(tool_name);
    let current_version = get_current_version_for_tool(tool_name);

    if !toolchains_dir.exists() {
        return Ok(InstalledVersionsReport {
            tool: tool_name.to_string(),
            current_version,
            versions: Vec::new(),
        });
    }

    let mut versions: Vec<String> = fs::read_dir(&toolchains_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    versions.sort();

    Ok(InstalledVersionsReport {
        tool: tool_name.to_string(),
        current_version: current_version.clone(),
        versions: versions
            .into_iter()
            .map(|version| InstalledVersionEntry {
                is_current: current_version.as_ref() == Some(&version),
                version,
            })
            .collect(),
    })
}

pub fn collect_remote(
    tool_name: &str,
    filter: RemoteFilter,
    use_cache: bool,
    show_spinner: bool,
) -> Result<RemoteVersionsReport> {
    let tool = tools::get_tool(tool_name)?;

    let mut versions = if show_spinner {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        spinner.set_message(format!("Fetching available versions of {}...", tool_name));
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        let versions = fetch_versions_cached(tool.as_ref(), use_cache)?;
        spinner.finish_and_clear();
        versions
    } else {
        fetch_versions_cached(tool.as_ref(), use_cache)?
    };

    let current_version = get_current_version_for_tool(tool_name);
    versions = apply_filter(tool_name, versions, filter);

    let latest_version = versions.first().map(|version| version.version.clone());
    let entries = versions
        .into_iter()
        .map(|version| {
            let normalized = version
                .version
                .strip_prefix('v')
                .unwrap_or(&version.version)
                .to_string();
            let is_current = current_version
                .as_ref()
                .map(|cv| cv == &normalized || cv == &version.version)
                .unwrap_or(false);
            let is_outdated = latest_version
                .as_ref()
                .map(|latest| is_version_outdated(&version.version, latest))
                .unwrap_or(false);

            RemoteVersionEntry {
                version: normalized,
                label: version.lts.clone(),
                is_current,
                is_outdated,
            }
        })
        .collect::<Vec<_>>();

    Ok(RemoteVersionsReport {
        tool: tool_name.to_string(),
        filter: remote_filter_name(filter).to_string(),
        total: entries.len(),
        current_version,
        versions: entries,
    })
}

fn render_installed_text(report: &InstalledVersionsReport) {
    if report.versions.is_empty() {
        println!("No versions of {} installed.", report.tool);
        return;
    }

    println!();
    println!("Installed versions of {}:", report.tool);
    println!();

    for version in &report.versions {
        if version.is_current {
            println!("  {} (current)", version.version);
        } else {
            println!("  {}", version.version);
        }
    }

    println!();
}

fn render_remote_text(report: &RemoteVersionsReport) {
    if report.versions.is_empty() {
        println!("{}", "No versions found matching the filter.".yellow());
        return;
    }

    println!();
    println!("{} {} versions:", "Available".cyan(), report.tool.yellow());
    println!();

    let mut count = 0;
    for version in &report.versions {
        let mut visible = version.version.clone();
        if let Some(label) = &version.label {
            if report.tool == "python" {
                visible.push_str(&format!(" (Status: {})", label));
            } else {
                visible.push_str(&format!(" (LTS: {})", label));
            }
        }
        if version.is_current {
            visible.push_str(" ← current");
        }

        let mut display = if version.is_current {
            format!("{}", version.version.green().bold())
        } else if version.is_outdated {
            format!("{}", version.version.dimmed())
        } else {
            version.version.clone()
        };

        if let Some(label) = &version.label {
            let label = if report.tool == "python" {
                format!("(Status: {})", label)
            } else {
                format!("(LTS: {})", label)
            };
            display.push_str(&format!(" {}", label.cyan()));
        }
        if version.is_current {
            display.push_str(&format!(" {}", "← current".green()));
        }

        let col_width = 28;
        let padding = if visible.len() < col_width {
            " ".repeat(col_width - visible.len())
        } else {
            "  ".to_string()
        };

        print!("  {}{}", display, padding);
        count += 1;
        if count % 3 == 0 {
            println!();
        }
    }
    if count % 3 != 0 {
        println!();
    }

    println!();
    println!(
        "{} {} versions (filter: {})",
        "Total:".dimmed(),
        report.total,
        report.filter.dimmed()
    );
}

fn apply_filter(tool_name: &str, versions: Vec<Version>, filter: RemoteFilter) -> Vec<Version> {
    match filter {
        RemoteFilter::All => versions,
        RemoteFilter::Lts => {
            if tool_name == "python" {
                Vec::new()
            } else {
                versions.into_iter().filter(|v| v.lts.is_some()).collect()
            }
        }
        RemoteFilter::Major => {
            let mut major_versions: HashMap<String, Vec<Version>> = HashMap::new();
            for version in versions {
                major_versions
                    .entry(extract_major_version(&version.version))
                    .or_default()
                    .push(version);
            }
            let mut result: Vec<_> = major_versions
                .into_values()
                .filter_map(|group| {
                    group
                        .into_iter()
                        .max_by_key(|version| version_sort_key(&version.version))
                })
                .collect();
            result.sort_by(|a, b| version_sort_key(&b.version).cmp(&version_sort_key(&a.version)));
            result
        }
        RemoteFilter::Latest => versions.into_iter().take(1).collect(),
    }
}

fn remote_filter_name(filter: RemoteFilter) -> &'static str {
    match filter {
        RemoteFilter::All => "all",
        RemoteFilter::Lts => "lts",
        RemoteFilter::Major => "major",
        RemoteFilter::Latest => "latest",
    }
}

fn extract_major_version(version: &str) -> String {
    let version = version.strip_prefix('v').unwrap_or(version);
    version.split('.').next().unwrap_or("0").to_string()
}

fn version_sort_key(version: &str) -> Vec<u32> {
    version
        .trim_start_matches('v')
        .split('.')
        .filter_map(|segment| segment.parse().ok())
        .collect()
}

fn is_version_outdated(version: &str, latest: &str) -> bool {
    let version_major = extract_major_version(version).parse::<i32>().unwrap_or(0);
    let latest_major = extract_major_version(latest).parse::<i32>().unwrap_or(0);
    version_major > 0 && latest_major > 0 && version_major < latest_major - 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_major_filter_keeps_newest_patch_per_major() {
        let filtered = apply_filter(
            "node",
            vec![
                Version {
                    version: "20.10.0".to_string(),
                    lts: None,
                },
                Version {
                    version: "20.9.0".to_string(),
                    lts: None,
                },
                Version {
                    version: "19.8.1".to_string(),
                    lts: None,
                },
                Version {
                    version: "19.8.0".to_string(),
                    lts: None,
                },
            ],
            RemoteFilter::Major,
        );

        let versions = filtered
            .into_iter()
            .map(|version| version.version)
            .collect::<Vec<_>>();
        assert_eq!(versions, vec!["20.10.0", "19.8.1"]);
    }
}
