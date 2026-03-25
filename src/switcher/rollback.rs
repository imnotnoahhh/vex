use super::links;
use crate::tools::Tool;
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;
use tracing::warn;

pub(super) fn current_version(base_dir: &Path, tool_name: &str) -> Option<String> {
    let current_link = base_dir.join("current").join(tool_name);
    if current_link.exists() {
        fs::read_link(&current_link).ok().and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
    } else {
        None
    }
}

pub(super) fn attempt_rollback(tool: &dyn Tool, base_dir: &Path, previous_version: Option<&str>) {
    let Some(previous_version) = previous_version else {
        return;
    };

    eprintln!(
        "{} Version switch failed, rolling back to {}...",
        "⚠".yellow(),
        previous_version
    );

    let previous_toolchain_dir = base_dir
        .join("toolchains")
        .join(tool.name())
        .join(previous_version);

    if !previous_toolchain_dir.exists() {
        warn!("Previous version {} no longer exists", previous_version);
        return;
    }

    match links::perform_switch(tool, base_dir, &previous_toolchain_dir) {
        Ok(_) => {
            eprintln!(
                "{} Rolled back to {}@{}",
                "✓".green(),
                tool.name(),
                previous_version
            );
        }
        Err(rollback_err) => {
            warn!("Rollback also failed: {}", rollback_err);
            eprintln!("{} Rollback failed: {}", "✗".red(), rollback_err);
        }
    }
}
