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
    summary.render();
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
