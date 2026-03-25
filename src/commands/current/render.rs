use super::CurrentReport;
use crate::ui;
use owo_colors::OwoColorize;

pub(super) fn render_text(report: &CurrentReport) {
    if report.tools.is_empty() {
        ui::dimmed("No tools activated yet.");
        println!();
        ui::dimmed("Use 'vex install <tool>' to install a tool.");
        return;
    }

    ui::header("Current active versions:");

    let mut table = ui::Table::new();
    for tool in &report.tools {
        let row = vec![
            tool.tool.yellow().to_string(),
            "→".to_string(),
            tool.version.cyan().to_string(),
            format!("({})", tool.source.dimmed()),
        ];
        table = table.row(row);

        if let Some(source_path) = &tool.source_path {
            table = table.row(vec![
                "".to_string(),
                "".to_string(),
                format!("{}: {}", "Source".dimmed(), source_path.dimmed()),
            ]);
        }
    }
    table.render();

    println!();
}
