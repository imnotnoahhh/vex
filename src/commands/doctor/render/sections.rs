use crate::commands::doctor::types::DoctorReport;
use crate::fs_utils::format_bytes;
use crate::ui;
use owo_colors::OwoColorize;

pub(super) fn render_sections(report: &DoctorReport) {
    render_disk_usage(report);
    render_unused_versions(report);
    render_lifecycle_warnings(report);
}

fn render_disk_usage(report: &DoctorReport) {
    if report.disk_usage.is_empty() {
        return;
    }

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

fn render_unused_versions(report: &DoctorReport) {
    if report.unused_versions.is_empty() {
        return;
    }

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

fn render_lifecycle_warnings(report: &DoctorReport) {
    if report.lifecycle_warnings.is_empty() {
        return;
    }

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
