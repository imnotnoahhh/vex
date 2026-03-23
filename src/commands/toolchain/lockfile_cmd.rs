use crate::error::{Result, VexError};
use crate::lockfile;
use crate::paths::vex_dir;
use crate::resolver;
use crate::tools::{self, Tool};
use owo_colors::OwoColorize;
use std::fs;

pub fn generate_lockfile() -> Result<()> {
    let cwd = resolver::current_dir();
    let versions = resolver::resolve_versions(&cwd);

    if versions.is_empty() {
        return Err(VexError::Config(
            "No version files found (.tool-versions, .node-version, etc.)".to_string(),
        ));
    }

    let mut lockfile = lockfile::Lockfile::new();

    for (tool_name, version) in &versions {
        let tool = match tools::get_tool(tool_name) {
            Ok(tool) => tool,
            Err(_) => {
                eprintln!("vex: skipping unsupported tool '{}'", tool_name);
                continue;
            }
        };

        let version_dir = vex_dir()?.join("toolchains").join(tool_name).join(version);
        if !version_dir.exists() {
            return Err(VexError::Config(format!(
                "Version {}@{} is not installed. Run 'vex install' first.",
                tool_name, version
            )));
        }

        let sha256 = get_installed_checksum(tool.as_ref(), version)?;
        lockfile.add_tool(
            tool_name.clone(),
            lockfile::LockEntry {
                version: version.clone(),
                sha256,
                url: None,
            },
        );
    }

    let path = lockfile.save_to_dir(&cwd)?;
    println!("{} Lockfile generated: {}", "✓".green(), path.display());
    println!();
    println!("{}", "Locked versions:".cyan().bold());
    for (tool, entry) in &lockfile.tools {
        println!("  {}@{}", tool.yellow(), entry.version.cyan());
    }

    Ok(())
}

fn get_installed_checksum(tool: &dyn Tool, version: &str) -> Result<Option<String>> {
    let vex = vex_dir()?;
    let checksum_file = vex
        .join("toolchains")
        .join(tool.name())
        .join(version)
        .join(".vex-checksum");

    if checksum_file.exists() {
        let content = fs::read_to_string(&checksum_file)?;
        Ok(Some(content.trim().to_string()))
    } else {
        Ok(None)
    }
}
