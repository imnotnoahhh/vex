use super::types::{CheckStatus, DoctorReport};
use owo_colors::OwoColorize;

pub(super) fn render_text(report: &DoctorReport) {
    println!();
    println!("{}", "vex doctor - Health Check".bold());
    println!();

    for check in &report.checks {
        let label = match check.status {
            CheckStatus::Ok => "✓".green().to_string(),
            CheckStatus::Warn => "⚠".yellow().to_string(),
            CheckStatus::Error => "✗".red().to_string(),
        };

        print!("Checking {}... ", check_display_name(&check.id));
        println!("{}", label);
        if check.status != CheckStatus::Ok {
            println!("  {}", check.summary.clone().yellow());
        }
        for detail in &check.details {
            println!("  {}", detail);
        }
    }

    println!();
    if report.issues == 0 && report.warnings == 0 {
        println!("{}", "✓ All checks passed!".green().bold());
    } else {
        if report.issues > 0 {
            println!("{} {} issue(s) found", "✗".red(), report.issues);
        }
        if report.warnings > 0 {
            println!("{} {} warning(s)", "⚠".yellow(), report.warnings);
        }
    }
    println!();
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
