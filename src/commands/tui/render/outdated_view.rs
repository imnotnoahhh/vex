use crate::commands::updates::OutdatedReport;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub(super) fn render(
    frame: &mut Frame,
    area: Rect,
    report: Option<&std::result::Result<OutdatedReport, String>>,
) {
    let block = Block::default()
        .title("Outdated Tools")
        .borders(Borders::ALL);

    match report {
        None => {
            let paragraph = Paragraph::new("Loading… (press 2 to refresh)").block(block);
            frame.render_widget(paragraph, area);
        }
        Some(Err(message)) => {
            let paragraph = Paragraph::new(Line::from(Span::styled(
                format!("Error: {}", message),
                Style::default().fg(Color::Red),
            )))
            .block(block);
            frame.render_widget(paragraph, area);
        }
        Some(Ok(report)) => {
            let items: Vec<ListItem> = if report.entries.is_empty() {
                vec![ListItem::new(Span::styled(
                    "All managed tools are up to date",
                    Style::default().fg(Color::Green),
                ))]
            } else {
                report
                    .entries
                    .iter()
                    .map(|entry| {
                        let status_color = match entry.status.as_str() {
                            "up_to_date" => Color::Green,
                            "outdated" => Color::Yellow,
                            _ => Color::Gray,
                        };
                        let mut spans = vec![
                            Span::styled(
                                format!("{:<10}", entry.tool),
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("{:<14}", entry.current_version),
                                Style::default().fg(Color::White),
                            ),
                            Span::raw(" → "),
                            Span::styled(
                                format!("{:<14}", entry.latest_version),
                                Style::default().fg(Color::Cyan),
                            ),
                            Span::styled(
                                format!(" [{}]", entry.status),
                                Style::default().fg(status_color),
                            ),
                        ];
                        if let Some(advisory) = &entry.advisory_status {
                            spans.push(Span::styled(
                                format!(" advisory:{}", advisory),
                                Style::default().fg(Color::Magenta),
                            ));
                        }
                        ListItem::new(Line::from(spans))
                    })
                    .collect()
            };

            let title = format!("Outdated Tools ({})", report.scope);
            let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
            frame.render_widget(list, area);
        }
    }
}
