use super::PruneReport;
use crate::fs_utils::format_bytes;
use owo_colors::OwoColorize;

pub(super) fn render_dry_run(report: &PruneReport) {
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

pub(super) fn render_completed(report: &PruneReport) {
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
