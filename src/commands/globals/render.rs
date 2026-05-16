use super::GlobalsReport;
use crate::ui;
use owo_colors::OwoColorize;

pub(super) fn render_text(report: &GlobalsReport, verbose: bool) {
    if report.entries.is_empty() {
        ui::dimmed("No global CLI entries detected.");
        println!();
        ui::dimmed(
            "Install a global CLI with shared npm globals, Go, Cargo, or 'vex python base pip'.",
        );
        return;
    }

    ui::header("Global CLIs and Build Tool State");
    let mut table = ui::Table::new();
    for entry in &report.entries {
        let version_context = entry
            .tool_version
            .as_ref()
            .map(|version| {
                format!(
                    "{} ({})",
                    version,
                    entry.version_source.as_deref().unwrap_or("unknown source")
                )
            })
            .unwrap_or_else(|| "n/a".to_string());
        table = table.row(vec![
            entry.tool.yellow().to_string(),
            entry.name.cyan().to_string(),
            entry.source.clone(),
            version_context.dimmed().to_string(),
        ]);
        if verbose {
            table = table.row(vec![
                "".to_string(),
                "".to_string(),
                format!("{}: {}", "Path".dimmed(), entry.path.dimmed()),
                entry
                    .version_source_path
                    .as_ref()
                    .map(|path| format!("{}: {}", "Version source".dimmed(), path.dimmed()))
                    .unwrap_or_default(),
            ]);
        }
    }
    table.render();
    if report
        .entries
        .iter()
        .any(|entry| entry.tool == "node" && entry.kind == "npm_global")
    {
        ui::dimmed(
            "Node npm globals are shared across vex-managed Node versions; project node_modules/.bin still wins when present.",
        );
    }
    println!();
}
