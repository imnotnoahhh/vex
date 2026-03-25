use crate::commands::updates::OutdatedReport;
use crate::ui;
use owo_colors::OwoColorize;

pub(in crate::commands::updates) fn render_outdated_text(report: &OutdatedReport) {
    if report.entries.is_empty() {
        ui::dimmed("No managed tools found in the current context.");
        return;
    }

    ui::header(&format!("Outdated check scope: {}", report.scope.cyan()));

    let mut table = ui::Table::new();
    let mut outdated_count = 0;
    let mut advisory_count = 0;

    for entry in &report.entries {
        let status = if entry.status == "outdated" {
            outdated_count += 1;
            "outdated".yellow().to_string()
        } else {
            "up to date".green().to_string()
        };

        let status_with_advisory = if let Some(advisory_status) = &entry.advisory_status {
            advisory_count += 1;
            match advisory_status.as_str() {
                "eol" => format!("{} {}", status, "(eol)".red()),
                "near_eol" => format!("{} {}", status, "(near eol)".yellow()),
                "lts_available" => format!("{} {}", status, "(lts available)".cyan()),
                "security_update_available" => {
                    format!("{} {}", status, "(security update)".yellow())
                }
                _ => status,
            }
        } else {
            status
        };

        table = table.row(vec![
            entry.tool.yellow().to_string(),
            entry.current_version.dimmed().to_string(),
            "→".to_string(),
            entry.latest_version.cyan().to_string(),
            format!("({})", status_with_advisory),
        ]);

        if let Some(message) = &entry.advisory_message {
            table = table.row(vec![
                "".to_string(),
                format!("{}: {}", "Advisory".yellow(), message.dimmed()),
            ]);
        }

        if let Some(recommendation) = &entry.advisory_recommendation {
            table = table.row(vec![
                "".to_string(),
                format!("{}: {}", "Recommendation".cyan(), recommendation.dimmed()),
            ]);
        }

        if let Some(path) = &entry.source_path {
            table = table.row(vec![
                "".to_string(),
                format!(
                    "{}: {} ({})",
                    "Source".dimmed(),
                    path.dimmed(),
                    super::source_label(entry.source).dimmed()
                ),
            ]);
        } else {
            table = table.row(vec![
                "".to_string(),
                format!(
                    "{}: {}",
                    "Source".dimmed(),
                    super::source_label(entry.source).dimmed()
                ),
            ]);
        }
    }

    table.render();

    println!();
    if outdated_count == 0 && advisory_count == 0 {
        ui::success("All managed tools are up to date.");
    } else {
        if outdated_count > 0 {
            ui::info(&format!(
                "{} tool(s) are behind the latest available version",
                outdated_count
            ));
        }
        if advisory_count > 0 {
            ui::warning(&format!(
                "{} tool(s) have lifecycle advisories",
                advisory_count
            ));
        }
    }
}
