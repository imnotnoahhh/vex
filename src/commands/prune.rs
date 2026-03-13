use crate::config;
use crate::error::{Result, VexError};
use crate::resolver;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const STALE_LOCK_AGE: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Serialize)]
pub struct RemovalCandidate {
    pub kind: String,
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct RetainedToolchain {
    pub tool: String,
    pub version: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct PruneReport {
    pub dry_run: bool,
    pub removable: Vec<RemovalCandidate>,
    pub retained_toolchains: Vec<RetainedToolchain>,
    pub total_candidates: usize,
    pub total_bytes: u64,
    pub note: String,
}

pub fn run(dry_run: bool) -> Result<()> {
    let report = collect_plan(dry_run)?;

    if dry_run {
        render_dry_run(&report);
        return Ok(());
    }

    for candidate in &report.removable {
        let path = PathBuf::from(&candidate.path);
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else if path.exists() {
            fs::remove_file(&path)?;
        }
    }

    render_completed(&report);
    Ok(())
}

fn collect_plan(dry_run: bool) -> Result<PruneReport> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let cwd = resolver::current_dir();
    collect_plan_for(&vex_dir, &cwd, dry_run, SystemTime::now())
}

fn collect_plan_for(
    vex_dir: &Path,
    cwd: &Path,
    dry_run: bool,
    now: SystemTime,
) -> Result<PruneReport> {
    let retained = retained_versions(vex_dir, cwd)?;
    let mut retained_toolchains = retained
        .iter()
        .map(|((tool, version), reason)| RetainedToolchain {
            tool: tool.clone(),
            version: version.clone(),
            reason: reason.clone(),
        })
        .collect::<Vec<_>>();
    retained_toolchains.sort_by(|a, b| a.tool.cmp(&b.tool).then(a.version.cmp(&b.version)));

    let mut removable = Vec::new();
    removable.extend(cache_candidates(vex_dir)?);
    removable.extend(stale_lock_candidates(vex_dir, now)?);
    removable.extend(unused_toolchain_candidates(vex_dir, &retained)?);

    let total_bytes = removable.iter().map(|item| item.bytes).sum();
    let total_candidates = removable.len();

    Ok(PruneReport {
        dry_run,
        removable,
        retained_toolchains,
        total_candidates,
        total_bytes,
        note: "Only ~/.vex state is pruned. Current activations, global defaults, and versions pinned in the current working tree are retained. Other repositories are not scanned.".to_string(),
    })
}

fn render_dry_run(report: &PruneReport) {
    println!();
    println!("{}", "vex prune --dry-run".bold());
    println!();

    if report.removable.is_empty() {
        println!("{}", "Nothing to prune.".green());
    } else {
        for item in &report.removable {
            println!(
                "  {} {} {} ({})",
                "→".cyan(),
                item.kind.yellow(),
                item.path.dimmed(),
                format_bytes(item.bytes).dimmed()
            );
        }
        println!();
        println!(
            "{} {} candidate(s), {} reclaimable",
            "Total:".bold(),
            report.total_candidates,
            format_bytes(report.total_bytes).cyan()
        );
    }

    if !report.retained_toolchains.is_empty() {
        println!();
        println!("{}", "Retained toolchains:".bold());
        for item in &report.retained_toolchains {
            println!(
                "  {} {}@{} ({})",
                "✓".green(),
                item.tool.yellow(),
                item.version.cyan(),
                item.reason.dimmed()
            );
        }
    }

    println!();
    println!("{}", report.note.dimmed());
    println!();
}

fn render_completed(report: &PruneReport) {
    println!();
    println!(
        "{} Removed {} item(s), reclaimed {}",
        "✓".green(),
        report.total_candidates,
        format_bytes(report.total_bytes).cyan()
    );
    println!("{}", report.note.dimmed());
    println!();
}

fn retained_versions(vex_dir: &Path, cwd: &Path) -> Result<HashMap<(String, String), String>> {
    let mut retained = HashMap::new();

    for (tool, version) in read_current_versions(vex_dir)? {
        retained
            .entry((tool, version))
            .or_insert_with(|| "active".to_string());
    }

    let global_path = vex_dir.join("tool-versions");
    for (tool, version) in read_tool_versions(&global_path) {
        retained
            .entry((tool, version))
            .or_insert_with(|| "global default".to_string());
    }

    for (tool, version) in resolve_project_versions(cwd) {
        retained
            .entry((tool, version))
            .or_insert_with(|| "current project".to_string());
    }

    Ok(retained)
}

fn cache_candidates(vex_dir: &Path) -> Result<Vec<RemovalCandidate>> {
    let cache_dir = vex_dir.join("cache");
    if !cache_dir.exists() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&cache_dir)?.filter_map(|e| e.ok()) {
        let path = entry.path();
        candidates.push(RemovalCandidate {
            kind: "cache".to_string(),
            bytes: path_size(&path),
            path: path.display().to_string(),
        });
    }

    Ok(candidates)
}

fn stale_lock_candidates(vex_dir: &Path, now: SystemTime) -> Result<Vec<RemovalCandidate>> {
    let locks_dir = vex_dir.join("locks");
    if !locks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&locks_dir)?.filter_map(|e| e.ok()) {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        let Ok(age) = now.duration_since(modified) else {
            continue;
        };
        if age >= STALE_LOCK_AGE {
            candidates.push(RemovalCandidate {
                kind: "stale_lock".to_string(),
                path: path.display().to_string(),
                bytes: metadata.len(),
            });
        }
    }

    Ok(candidates)
}

fn unused_toolchain_candidates(
    vex_dir: &Path,
    retained: &HashMap<(String, String), String>,
) -> Result<Vec<RemovalCandidate>> {
    let toolchains_dir = vex_dir.join("toolchains");
    if !toolchains_dir.exists() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for tool_entry in fs::read_dir(&toolchains_dir)?.filter_map(|e| e.ok()) {
        if !tool_entry
            .file_type()
            .ok()
            .map(|ft| ft.is_dir())
            .unwrap_or(false)
        {
            continue;
        }
        let tool = tool_entry.file_name().to_string_lossy().to_string();
        for version_entry in fs::read_dir(tool_entry.path())?.filter_map(|e| e.ok()) {
            if !version_entry
                .file_type()
                .ok()
                .map(|ft| ft.is_dir())
                .unwrap_or(false)
            {
                continue;
            }

            let version = version_entry.file_name().to_string_lossy().to_string();
            if retained.contains_key(&(tool.clone(), version.clone())) {
                continue;
            }

            let path = version_entry.path();
            candidates.push(RemovalCandidate {
                kind: "toolchain".to_string(),
                bytes: path_size(&path),
                path: path.display().to_string(),
            });
        }
    }

    Ok(candidates)
}

fn read_current_versions(vex_dir: &Path) -> Result<HashMap<String, String>> {
    let current_dir = vex_dir.join("current");
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
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect()
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

fn path_size(path: &Path) -> u64 {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return 0;
    };
    let file_type = metadata.file_type();

    if file_type.is_symlink() {
        return 0;
    }

    if metadata.is_file() {
        return metadata.len();
    }

    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| path_size(&entry.path()))
        .sum()
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.2} GiB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.2} MiB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.2} KiB", bytes / KB)
    } else {
        format!("{} B", bytes as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn test_path_size_skips_symlinked_directories() {
        let dir = tempfile::tempdir().unwrap();
        let real_dir = dir.path().join("real");
        let linked_dir = dir.path().join("linked");
        fs::create_dir_all(&real_dir).unwrap();
        fs::write(real_dir.join("payload.bin"), vec![0_u8; 16]).unwrap();
        std::os::unix::fs::symlink(&real_dir, &linked_dir).unwrap();

        assert_eq!(path_size(dir.path()), 16);
    }
}
