use crate::commands::updates::UpgradeReport;
use crate::ui;
use owo_colors::OwoColorize;

pub(in crate::commands::updates) fn render_upgrade_text(report: &UpgradeReport) {
    if report.entries.is_empty() {
        ui::dimmed("No managed tools found to upgrade.");
        return;
    }

    ui::header(&format!("Upgrade scope: {}", report.scope.cyan()));

    let mut summary = ui::Summary::new();

    for entry in &report.entries {
        let message = format!(
            "{}  {} → {}",
            entry.tool.yellow(),
            entry.previous_version.dimmed(),
            entry.target_version.cyan()
        );

        match entry.status.as_str() {
            "already_latest" => summary = summary.info(message),
            "upgraded" => {
                summary = summary.success(message);
                if let Some(path) = &entry.source_path {
                    summary = summary.info(format!(
                        "  Updated: {} ({})",
                        path.dimmed(),
                        super::source_label(entry.source).dimmed()
                    ));
                }
            }
            _ => summary = summary.info(message),
        }
    }

    summary.render();
}
