mod layout;
mod widgets;

use super::state::DashboardState;
use crate::error::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use std::io;

use layout::split_dashboard;
use widgets::{
    render_current_versions, render_disk_usage, render_footer, render_header,
    render_managed_versions, render_missing_installs, render_warnings,
};

pub(super) fn run_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: DashboardState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| render_dashboard(frame, &state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                _ => {}
            }
        }
    }
}

fn render_dashboard(frame: &mut Frame, state: &DashboardState) {
    let layout = split_dashboard(frame.area());

    render_header(frame, layout.header);
    render_current_versions(frame, layout.current_versions, &state.current_tools);
    render_managed_versions(frame, layout.managed_versions, &state.managed_versions);
    render_missing_installs(frame, layout.missing_installs, &state.missing_installs);
    render_warnings(frame, layout.warnings, &state.warnings);
    render_disk_usage(frame, layout.disk_usage, &state.disk_usage);
    render_footer(frame, layout.footer);
}
