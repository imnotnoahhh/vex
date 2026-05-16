mod doctor_view;
mod layout;
mod outdated_view;
mod widgets;

use super::state::{AppState, View};
use crate::error::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use std::io;

use layout::split_dashboard;
use widgets::{
    render_current_versions, render_disk_usage, render_footer, render_managed_versions,
    render_missing_installs, render_warnings,
};

pub(super) fn run_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut state: AppState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| render_app(frame, &state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('1') => state.view = View::Dashboard,
                KeyCode::Char('2') => {
                    state.view = View::Outdated;
                    state.ensure_outdated();
                }
                KeyCode::Char('3') => {
                    state.view = View::Doctor;
                    state.ensure_doctor();
                }
                KeyCode::Tab => match state.view {
                    View::Dashboard => {
                        state.view = View::Outdated;
                        state.ensure_outdated();
                    }
                    View::Outdated => {
                        state.view = View::Doctor;
                        state.ensure_doctor();
                    }
                    View::Doctor => state.view = View::Dashboard,
                },
                KeyCode::BackTab => match state.view {
                    View::Dashboard => {
                        state.view = View::Doctor;
                        state.ensure_doctor();
                    }
                    View::Outdated => state.view = View::Dashboard,
                    View::Doctor => {
                        state.view = View::Outdated;
                        state.ensure_outdated();
                    }
                },
                KeyCode::Char('r') => match state.view {
                    View::Outdated => {
                        state.outdated = None;
                        state.ensure_outdated();
                    }
                    View::Doctor => {
                        state.doctor = None;
                        state.ensure_doctor();
                    }
                    View::Dashboard => {}
                },
                _ => {}
            }
        }
    }
}

fn render_app(frame: &mut Frame, state: &AppState) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_tabs(frame, outer[0], state.view);
    match state.view {
        View::Dashboard => render_dashboard(frame, outer[1], state),
        View::Outdated => outdated_view::render(frame, outer[1], state.outdated.as_ref()),
        View::Doctor => doctor_view::render(frame, outer[1], state.doctor.as_ref()),
    }
    render_footer(frame, outer[2]);
}

fn render_tabs(frame: &mut Frame, area: Rect, current: View) {
    let titles = [View::Dashboard, View::Outdated, View::Doctor];
    let mut spans = vec![Span::styled(
        "vex TUI ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )];
    for (idx, view) in titles.iter().enumerate() {
        let label = format!(" {}:{} ", idx + 1, view.title());
        let style = if *view == current {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(label, style));
        spans.push(Span::raw(" "));
    }
    let header = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}

fn render_dashboard(frame: &mut Frame, area: Rect, state: &AppState) {
    let layout = split_dashboard(area);

    render_current_versions(
        frame,
        layout.current_versions,
        &state.dashboard.current_tools,
    );
    render_managed_versions(
        frame,
        layout.managed_versions,
        &state.dashboard.managed_versions,
    );
    render_missing_installs(
        frame,
        layout.missing_installs,
        &state.dashboard.missing_installs,
    );
    render_warnings(frame, layout.warnings, &state.dashboard.warnings);
    render_disk_usage(frame, layout.disk_usage, &state.dashboard.disk_usage);
}
