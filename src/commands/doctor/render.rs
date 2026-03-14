use super::types::{CheckStatus, DoctorReport};
use crate::ui;
use owo_colors::OwoColorize;

pub(super) fn render_text(report: &DoctorReport) {
    ui::header("vex doctor - Health Check");

    for check in &report.checks {
        let check_name = check_display_name(&check.id);

        match check.status {
            CheckStatus::Ok => {
                ui::success(&format!("Checking {}... passed", check_name));
            }
            CheckStatus::Warn => {
                ui::warning(&format!(
                    "Checking {}... {}",
                    check_name,
                    check.summary.yellow()
                ));
                for detail in &check.details {
                    println!("  {}", detail);
                }
            }
            CheckStatus::Error => {
                ui::error(&format!("Checking {}... {}", check_name, check.summary));
                for detail in &check.details {
                    println!("  {}", detail);
                }
            }
        }
    }

    println!();

    // Render disk usage section
    if !report.disk_usage.is_empty() {
        ui::header("Disk Usage by Tool");
        let mut table = ui::Table::new();
        for usage in &report.disk_usage {
            table = table.row(vec![
                usage.tool.yellow().to_string(),
                format!("{} version(s)", usage.version_count),
                format_bytes(usage.total_bytes).cyan().to_string(),
            ]);
        }
        table.render();
        println!();
        println!(
            "  {} {}",
            "Total disk usage:".bold(),
            format_bytes(report.total_disk_bytes).cyan()
        );
        println!();
    }

    // Render unused versions section
    if !report.unused_versions.is_empty() {
        ui::header("Unused Versions");
        let mut table = ui::Table::new();
        for unused in report.unused_versions.iter().take(10) {
            table = table.row(vec![
                unused.tool.yellow().to_string(),
                unused.version.dimmed().to_string(),
                format_bytes(unused.bytes).cyan().to_string(),
            ]);
        }
        table.render();
        println!();
        if report.unused_versions.len() > 10 {
            println!(
                "  {} (showing 10 of {})",
                "...".dimmed(),
                report.unused_versions.len()
            );
            println!();
        }
        println!(
            "  {} {}",
            "Reclaimable space:".bold(),
            format_bytes(report.reclaimable_bytes).cyan()
        );
        println!();
    }

    // Render lifecycle warnings section
    if !report.lifecycle_warnings.is_empty() {
        ui::header("Lifecycle Warnings");
        for warning in &report.lifecycle_warnings {
            let (status_icon, status_color) = match warning.status.as_str() {
                "eol" => ("✗", "red"),
                "near_eol" => ("⚠", "yellow"),
                "outdated" => ("→", "yellow"),
                _ => ("→", "cyan"),
            };
            let colored_icon = match status_color {
                "red" => status_icon.red().to_string(),
                "yellow" => status_icon.yellow().to_string(),
                _ => status_icon.cyan().to_string(),
            };
            println!(
                "  {} {}@{} - {}",
                colored_icon,
                warning.tool.yellow(),
                warning.version.dimmed(),
                warning.message
            );
        }
        println!();
    }

    // Render summary
    let mut summary = ui::Summary::new();
    if report.issues == 0 && report.warnings == 0 {
        summary = summary.success("All checks passed!".to_string());
    } else {
        if report.issues > 0 {
            summary = summary.error(format!("{} issue(s) found", report.issues));
        }
        if report.warnings > 0 {
            summary = summary.warning(format!("{} warning(s)", report.warnings));
        }
    }

    // Add suggestions
    if !report.suggestions.is_empty() {
        for suggestion in &report.suggestions {
            summary = summary.info(suggestion.clone());
        }
    }

    summary.render();
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

fn check_display_name(id: &str) -> &'static str {
    match id {
        "vex_directory" => "vex directory",
        "directory_structure" => "directory structure",
        "path" => "PATH configuration",
        "path_priority" => "PATH priority",
        "shell_hook" => "shell hook",
        "config" => "config file",
        "global_tool_versions" => "global tool-versions",
        "project_config" => "project config",
        "effective_settings" => "effective settings",
        "shell_hook_duplicates" => "shell hook duplicates",
        "installed_tools" => "installed tools",
        "symlinks" => "symlinks integrity",
        "binary_permissions" => "binary executability",
        "binary_runnability" => "binary runnability",
        "cache_integrity" => "cache integrity",
        "network" => "network connectivity",
        _ => "health check",
    }
}
