use crate::commands::doctor::{CheckStatus, DoctorReport};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub(super) fn render(
    frame: &mut Frame,
    area: Rect,
    report: Option<&std::result::Result<DoctorReport, String>>,
) {
    match report {
        None => {
            let paragraph = Paragraph::new("Loading… (press 3 to refresh)")
                .block(Block::default().title("Doctor").borders(Borders::ALL));
            frame.render_widget(paragraph, area);
        }
        Some(Err(message)) => {
            let paragraph = Paragraph::new(Line::from(Span::styled(
                format!("Error: {}", message),
                Style::default().fg(Color::Red),
            )))
            .block(Block::default().title("Doctor").borders(Borders::ALL));
            frame.render_widget(paragraph, area);
        }
        Some(Ok(report)) => render_report(frame, area, report),
    }
}

fn render_report(frame: &mut Frame, area: Rect, report: &DoctorReport) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    let summary = format!(
        "root: {}\nissues: {}   warnings: {}\ntotal disk: {} MB   reclaimable: {} MB",
        report.root,
        report.issues,
        report.warnings,
        report.total_disk_bytes / 1024 / 1024,
        report.reclaimable_bytes / 1024 / 1024,
    );
    let summary_block = Paragraph::new(summary).block(
        Block::default()
            .title("Doctor Summary")
            .borders(Borders::ALL),
    );
    frame.render_widget(summary_block, chunks[0]);

    let check_items: Vec<ListItem> = if report.checks.is_empty() {
        vec![ListItem::new("No checks recorded")]
    } else {
        report
            .checks
            .iter()
            .map(|check| {
                let (icon, color) = match check.status {
                    CheckStatus::Ok => ("OK   ", Color::Green),
                    CheckStatus::Warn => ("WARN ", Color::Yellow),
                    CheckStatus::Error => ("ERROR", Color::Red),
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        icon,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:<28}", check.id),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(check.summary.clone()),
                ]))
            })
            .collect()
    };
    let checks =
        List::new(check_items).block(Block::default().title("Checks").borders(Borders::ALL));
    frame.render_widget(checks, chunks[1]);

    let suggestion_items: Vec<ListItem> = if report.suggestions.is_empty() {
        vec![ListItem::new(Span::styled(
            "No suggestions",
            Style::default().fg(Color::Green),
        ))]
    } else {
        report
            .suggestions
            .iter()
            .map(|s| ListItem::new(Span::styled(s.as_str(), Style::default().fg(Color::Yellow))))
            .collect()
    };
    let suggestions = List::new(suggestion_items)
        .block(Block::default().title("Suggestions").borders(Borders::ALL));
    frame.render_widget(suggestions, chunks[2]);
}
