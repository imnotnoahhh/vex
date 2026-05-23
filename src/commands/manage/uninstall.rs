use crate::alias::AliasManager;
use crate::error::{Result, VexError};
use crate::paths::vex_dir;
use crate::spec::parse_spec;
use crate::tools;
use crate::version_files;
use owo_colors::OwoColorize;
use std::fs;

pub fn uninstall_spec(spec: &str) -> Result<()> {
    let (tool_name, version) = parse_spec(spec)?;
    if version.is_empty() {
        return Err(VexError::Parse(
            "Please specify a version to uninstall (e.g., node@20.11.0)".to_string(),
        ));
    }

    uninstall(&tool_name, &version)
}

pub fn uninstall(tool_name: &str, version: &str) -> Result<()> {
    let vex_dir = vex_dir()?;
    let version_dir = vex_dir.join("toolchains").join(tool_name).join(version);
    if !version_dir.exists() {
        return Err(VexError::VersionNotFound {
            tool: tool_name.to_string(),
            version: version.to_string(),
            suggestions: String::new(),
        });
    }

    println!("Uninstalling {} {}...", tool_name, version);

    let is_active = active_version_matches(&vex_dir, tool_name, &version_dir);
    fs::remove_dir_all(&version_dir)?;

    if is_active {
        remove_active_links(&vex_dir, tool_name)?;
    }

    prune_dangling_references(&vex_dir, tool_name, version)?;

    println!(
        "{} Uninstalled {} {}",
        "✓".green(),
        tool_name.yellow(),
        version.yellow()
    );
    Ok(())
}

/// Remove vex-owned references to the just-uninstalled `tool@version`.
///
/// Touches only files vex itself manages:
/// - `~/.vex/aliases.toml` global aliases
/// - `~/.vex/tool-versions` global pin
///
/// Project-level `.tool-versions` lives under user/team control, so we only warn
/// when one nearby still pins the removed version.
fn prune_dangling_references(
    vex_dir: &std::path::Path,
    tool_name: &str,
    version: &str,
) -> Result<()> {
    let manager = AliasManager::new(vex_dir);
    let removed_aliases = manager.prune_global_for_version(tool_name, version)?;
    for alias in &removed_aliases {
        println!(
            "  {} dropped global alias {}@{}",
            "·".dimmed(),
            tool_name,
            alias
        );
    }

    let global_pin = vex_dir.join("tool-versions");
    if version_files::remove_tool_version_if_matches(&global_pin, tool_name, version)? {
        println!(
            "  {} cleared {} from {}",
            "·".dimmed(),
            tool_name,
            global_pin.display().to_string().dimmed()
        );
    }

    if let Some(local) = find_local_tool_version_pin(
        &std::env::current_dir().unwrap_or_default(),
        tool_name,
        version,
    ) {
        eprintln!(
            "  {} {} still pins {}@{} — leaving it alone (project files are user-owned)",
            "!".yellow(),
            local.display(),
            tool_name,
            version
        );
    }

    Ok(())
}

fn find_local_tool_version_pin(
    start_dir: &std::path::Path,
    tool_name: &str,
    version: &str,
) -> Option<std::path::PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(".tool-versions");
        if candidate.is_file() {
            if let Ok(content) = std::fs::read_to_string(&candidate) {
                if content.lines().any(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        return false;
                    }
                    let mut parts = trimmed.split_whitespace();
                    parts.next() == Some(tool_name) && parts.next() == Some(version)
                }) {
                    return Some(candidate);
                }
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn active_version_matches(
    vex_dir: &std::path::Path,
    tool_name: &str,
    version_dir: &std::path::Path,
) -> bool {
    let current_link = vex_dir.join("current").join(tool_name);
    current_link.exists()
        && fs::read_link(&current_link)
            .map(|target| target == version_dir)
            .unwrap_or(false)
}

fn remove_active_links(vex_dir: &std::path::Path, tool_name: &str) -> Result<()> {
    let current_link = vex_dir.join("current").join(tool_name);
    let _ = fs::remove_file(&current_link);

    let tool = tools::get_tool(tool_name)?;
    let bin_dir = vex_dir.join("bin");
    for (bin_name, _) in tool.bin_paths() {
        let _ = fs::remove_file(bin_dir.join(bin_name));
    }

    Ok(())
}
