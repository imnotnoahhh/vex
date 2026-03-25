use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(super) struct DashboardLayout {
    pub(super) header: Rect,
    pub(super) current_versions: Rect,
    pub(super) managed_versions: Rect,
    pub(super) missing_installs: Rect,
    pub(super) warnings: Rect,
    pub(super) disk_usage: Rect,
    pub(super) footer: Rect,
}

pub(super) fn split_dashboard(area: Rect) -> DashboardLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(main_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_chunks[1]);

    DashboardLayout {
        header: chunks[0],
        current_versions: left_chunks[0],
        managed_versions: left_chunks[1],
        missing_installs: left_chunks[2],
        warnings: right_chunks[0],
        disk_usage: right_chunks[1],
        footer: chunks[2],
    }
}
