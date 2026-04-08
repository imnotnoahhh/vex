//! Version switching module
//!
//! Implements tool version switching via atomic symlink updates.
//! Updates `~/.vex/current/<tool>` and executable links in `~/.vex/bin/`.

mod failure;
mod links;
mod rollback;

use crate::error::{Result, VexError};
use crate::paths::vex_dir;
use crate::tools::Tool;
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

use failure::maybe_fail_bin_link;
#[cfg(test)]
use failure::{inject_test_failure, TestFailurePoint};
use rollback::{attempt_rollback, current_version};

/// Switch tool to specified version
///
/// Atomically updates `~/.vex/current/<tool>` symlink and executable links in `~/.vex/bin/`.
pub fn switch_version(tool: &dyn Tool, version: &str) -> Result<()> {
    info!("Switching version: {}@{}", tool.name(), version);
    switch_version_in(tool, version, &vex_dir()?)
}

pub fn relink_current_tool(tool: &dyn Tool) -> Result<()> {
    relink_current_tool_in(tool, &vex_dir()?)
}

fn switch_version_in(tool: &dyn Tool, version: &str, base_dir: &Path) -> Result<()> {
    debug!("Switch version in base_dir: {}", base_dir.display());
    let toolchain_dir = base_dir.join("toolchains").join(tool.name()).join(version);

    if !toolchain_dir.exists() {
        return Err(VexError::VersionNotFound {
            tool: tool.name().to_string(),
            version: version.to_string(),
            suggestions: String::new(),
        });
    }

    let old_version = current_version(base_dir, tool.name());
    debug!("Current version: {:?}", old_version);

    println!(
        "{} {} to version {}...",
        "Switching".cyan(),
        tool.name().yellow(),
        version.yellow()
    );

    match links::perform_switch(tool, base_dir, &toolchain_dir) {
        Ok(_) => {
            println!("{} Switched to {}@{}", "✓".green(), tool.name(), version);
            Ok(())
        }
        Err(err) => {
            warn!("Version switch failed: {}, attempting rollback", err);
            attempt_rollback(tool, base_dir, old_version.as_deref());
            Err(err)
        }
    }
}

fn relink_current_tool_in(tool: &dyn Tool, base_dir: &Path) -> Result<()> {
    let current_link = base_dir.join("current").join(tool.name());
    if !current_link.exists() {
        return Err(VexError::Parse(format!(
            "No active {} version found. Run 'vex use {}@<version>' first.",
            tool.name(),
            tool.name()
        )));
    }

    let toolchain_dir = fs::read_link(&current_link)?;
    links::rebuild_bin_links(tool, base_dir, &toolchain_dir)
}

#[cfg(test)]
mod tests;
