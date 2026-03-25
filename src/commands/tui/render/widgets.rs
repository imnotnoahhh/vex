use super::super::state::DiskUsage;
use crate::commands::current::CurrentEntry;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashMap;

pub(super) fn render_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new("vex TUI Dashboard")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}

pub(super) fn render_current_versions(frame: &mut Frame, area: Rect, tools: &[CurrentEntry]) {
    let items = if tools.is_empty() {
        vec![ListItem::new("No tools activated yet")]
    } else {
        tools
            .iter()
            .map(|entry| {
                ListItem::new(Line::from(vec![
                    Span::styled(&entry.tool, Style::default().fg(Color::Yellow)),
                    Span::raw(" -> "),
                    Span::styled(&entry.version, Style::default().fg(Color::Cyan)),
                    Span::raw(" ("),
                    Span::styled(&entry.source, Style::default().fg(Color::DarkGray)),
                    Span::raw(")"),
                ]))
            })
            .collect()
    };

    render_list_block(frame, area, "Current Active Versions", items);
}

pub(super) fn render_managed_versions(
    frame: &mut Frame,
    area: Rect,
    versions: &HashMap<String, String>,
) {
    let mut sorted_versions: Vec<(&String, &String)> = versions.iter().collect();
    sorted_versions.sort_by(|a, b| a.0.cmp(b.0));

    let items = if sorted_versions.is_empty() {
        vec![ListItem::new("No managed versions")]
    } else {
        sorted_versions
            .iter()
            .map(|(tool, version)| {
                ListItem::new(Line::from(vec![
                    Span::styled(tool.as_str(), Style::default().fg(Color::Green)),
                    Span::raw(" -> "),
                    Span::styled(version.as_str(), Style::default().fg(Color::White)),
                ]))
            })
            .collect()
    };

    render_list_block(frame, area, "Managed Versions", items);
}

pub(super) fn render_missing_installs(frame: &mut Frame, area: Rect, missing: &[String]) {
    let items = if missing.is_empty() {
        vec![ListItem::new(Span::styled(
            "All managed versions installed",
            Style::default().fg(Color::Green),
        ))]
    } else {
        missing
            .iter()
            .map(|spec| ListItem::new(Span::styled(spec.as_str(), Style::default().fg(Color::Red))))
            .collect()
    };

    render_list_block(frame, area, "Missing Installations", items);
}

pub(super) fn render_warnings(frame: &mut Frame, area: Rect, warnings: &[String]) {
    let items = if warnings.is_empty() {
        vec![ListItem::new(Span::styled(
            "No warnings",
            Style::default().fg(Color::Green),
        ))]
    } else {
        warnings
            .iter()
            .map(|warning| {
                ListItem::new(Span::styled(
                    warning.as_str(),
                    Style::default().fg(Color::Yellow),
                ))
            })
            .collect()
    };

    render_list_block(frame, area, "Health Warnings", items);
}

pub(super) fn render_disk_usage(frame: &mut Frame, area: Rect, disk_usage: &Option<DiskUsage>) {
    let text = if let Some(usage) = disk_usage {
        format!(
            "vex: {} MB\nAvailable: {} MB",
            usage.vex_size_mb, usage.available_mb
        )
    } else {
        "Unable to calculate disk usage".to_string()
    };

    let paragraph =
        Paragraph::new(text).block(Block::default().title("Disk Usage").borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

pub(super) fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new("Press 'q' or ESC to exit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

fn render_list_block(frame: &mut Frame, area: Rect, title: &str, items: Vec<ListItem>) {
    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(list, area);
}
