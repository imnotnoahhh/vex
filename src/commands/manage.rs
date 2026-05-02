mod set;
mod uninstall;

use crate::advisories;
use crate::error::{Result, VexError};
use crate::paths::vex_dir;
use crate::requested_versions;
use crate::spec::parse_spec;
use crate::switcher;
use crate::tools;
use owo_colors::OwoColorize;

pub use set::{set_global_version, set_project_version};
pub use uninstall::uninstall_spec;

pub fn relink_tool(tool_name: &str) -> Result<()> {
    if tool_name != "node" {
        return Err(VexError::Parse(
            "'vex relink' currently supports node only. Try 'vex relink node'.".to_string(),
        ));
    }

    let tool = tools::get_tool(tool_name)?;
    switcher::relink_current_tool(tool.as_ref())?;
    println!(
        "{} Rebuilt managed binary links for {}",
        "✓".green(),
        tool_name.yellow()
    );
    Ok(())
}

pub fn use_spec(spec: &str) -> Result<()> {
    let (tool_name, version) = parse_spec(spec)?;
    if version.is_empty() {
        return Err(VexError::Parse(
            "Please specify a version (e.g., node@20.11.0) or use --auto".to_string(),
        ));
    }

    let tool = tools::get_tool(&tool_name)?;
    let vex = vex_dir()?;
    let resolved = match requested_versions::resolve_installed_version(&vex, &tool_name, &version)?
    {
        Some(installed) => installed,
        None => tools::resolve_fuzzy_version(tool.as_ref(), &version)?,
    };
    switcher::switch_version(tool.as_ref(), &resolved)?;

    let advisory = advisories::get_advisory(&tool_name, &resolved);
    if advisory.is_warning() {
        println!();
        if let Some(msg) = &advisory.message {
            println!("{} {}", "warning:".yellow().bold(), msg);
        }
        if let Some(rec) = &advisory.recommendation {
            println!("{} {}", "recommendation:".cyan(), rec);
        }
    }

    Ok(())
}
#[cfg(test)]
mod tests;
