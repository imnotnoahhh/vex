use crate::error::{Result, VexError};
use crate::paths::vex_dir;
use crate::spec::parse_spec;
use crate::tools;
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

    println!(
        "{} Uninstalled {} {}",
        "✓".green(),
        tool_name.yellow(),
        version.yellow()
    );
    Ok(())
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
