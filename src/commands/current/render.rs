use super::CurrentReport;
use crate::ui;
use owo_colors::OwoColorize;

pub(super) fn render_text(report: &CurrentReport, verbose: bool) {
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

        if verbose {
            if let Some(metadata) = &tool.metadata {
                table = table.row(vec![
                    "".to_string(),
                    "".to_string(),
                    format!(
                        "{}: {}",
                        "Source URL".dimmed(),
                        metadata
                            .provenance
                            .source_url
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string())
                            .dimmed()
                    ),
                ]);
                if !metadata.extensions.is_empty() {
                    table = table.row(vec![
                        "".to_string(),
                        "".to_string(),
                        format!(
                            "{}: {}",
                            "Extensions".dimmed(),
                            metadata
                                .extensions
                                .iter()
                                .map(|extension| extension.name.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                                .dimmed()
                        ),
                    ]);
                }
            }
        }
    }
    table.render();

    println!();
}
