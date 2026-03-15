use crate::advisories;
use crate::commands::current::{collect_current, CurrentEntry};
use crate::config;
use crate::error::{Result, VexError};
use crate::resolver;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
struct DashboardState {
    current_tools: Vec<CurrentEntry>,
    warnings: Vec<String>,
    disk_usage: Option<DiskUsage>,
    managed_versions: HashMap<String, String>,
    missing_installs: Vec<String>,
}

#[derive(Debug)]
struct DiskUsage {
    vex_size_mb: u64,
    available_mb: u64,
}

pub fn run() -> Result<()> {
    // Check if we're in an interactive terminal
    if !atty::is(atty::Stream::Stdout) {
        return Err(VexError::Dialog(
            "TUI requires an interactive terminal".to_string(),
        ));
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Collect dashboard data
    let state = collect_dashboard_state()?;

    // Run the TUI
    let res = run_tui(&mut terminal, state);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn collect_dashboard_state() -> Result<DashboardState> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let pwd = resolver::current_dir();

    // Collect current active versions
    let current_report = collect_current()?;
    let current_tools = current_report.tools;

    // Collect managed versions from project and global contexts
    let versions = resolver::resolve_versions(&pwd);
    let global_path = vex_dir.join("tool-versions");
    let global_versions = read_tool_versions(&global_path);

    let mut managed_versions = HashMap::new();
    managed_versions.extend(global_versions);
    managed_versions.extend(versions);

    // Find missing installations
    let mut missing_installs = Vec::new();
    for (tool, version) in &managed_versions {
        let tool_dir = vex_dir.join("toolchains").join(tool).join(version);
        if !tool_dir.exists() {
            missing_installs.push(format!("{}@{}", tool, version));
        }
    }

    // Collect warnings (EOL, outdated, etc.)
    let mut warnings = Vec::new();
    for entry in &current_tools {
        let advisory = advisories::get_advisory(&entry.tool, &entry.version);
        if advisory.is_warning() {
            if let Some(msg) = advisory.message {
                warnings.push(format!("{}: {}", entry.tool, msg));
            }
        }
    }

    // Calculate disk usage
    let disk_usage = calculate_disk_usage(&vex_dir);

    Ok(DashboardState {
        current_tools,
        warnings,
        disk_usage,
        managed_versions,
        missing_installs,
    })
}

fn calculate_disk_usage(vex_dir: &Path) -> Option<DiskUsage> {
    let toolchains_dir = vex_dir.join("toolchains");
    let cache_dir = vex_dir.join("cache");

    let mut total_size = 0u64;

    // Calculate toolchains size
    if let Ok(entries) = fs::read_dir(&toolchains_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(size) = dir_size(&entry.path()) {
                total_size += size;
            }
        }
    }

    // Calculate cache size
    if let Ok(size) = dir_size(&cache_dir) {
        total_size += size;
    }

    // Get available disk space
    let available_mb = if fs::metadata(vex_dir).is_ok() {
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        disks
            .iter()
            .find(|d| vex_dir.starts_with(d.mount_point()))
            .map(|d| d.available_space() / 1024 / 1024)
            .unwrap_or(0)
    } else {
        0
    };

    Some(DiskUsage {
        vex_size_mb: total_size / 1024 / 1024,
        available_mb,
    })
}

fn dir_size(path: &Path) -> io::Result<u64> {
    let mut size = 0u64;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                size += dir_size(&entry.path())?;
            } else {
                size += metadata.len();
            }
        }
    } else {
        size = fs::metadata(path)?.len();
    }

    Ok(size)
}

fn read_tool_versions(path: &Path) -> HashMap<String, String> {
    let Ok(content) = fs::read_to_string(path) else {
        return HashMap::new();
    };

    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }

            let mut parts = line.split_whitespace();
            let tool = parts.next()?;
            let version = parts.next()?;
            Some((tool.to_string(), version.to_string()))
        })
        .collect()
}

fn run_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: DashboardState,
) -> Result<()> {
    loop {
        terminal.draw(|f| render_dashboard(f, &state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

fn render_dashboard(f: &mut Frame, state: &DashboardState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Render header
    render_header(f, chunks[0]);

    // Split main content into sections
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

    // Render sections
    render_current_versions(f, left_chunks[0], &state.current_tools);
    render_managed_versions(f, left_chunks[1], &state.managed_versions);
    render_missing_installs(f, left_chunks[2], &state.missing_installs);
    render_warnings(f, right_chunks[0], &state.warnings);
    render_disk_usage(f, right_chunks[1], &state.disk_usage);

    // Render footer
    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new("vex TUI Dashboard")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

fn render_current_versions(f: &mut Frame, area: Rect, tools: &[CurrentEntry]) {
    let items: Vec<ListItem> = if tools.is_empty() {
        vec![ListItem::new("No tools activated yet")]
    } else {
        tools
            .iter()
            .map(|entry| {
                let line = Line::from(vec![
                    Span::styled(&entry.tool, Style::default().fg(Color::Yellow)),
                    Span::raw(" → "),
                    Span::styled(&entry.version, Style::default().fg(Color::Cyan)),
                    Span::raw(" ("),
                    Span::styled(&entry.source, Style::default().fg(Color::DarkGray)),
                    Span::raw(")"),
                ]);
                ListItem::new(line)
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .title("Current Active Versions")
            .borders(Borders::ALL),
    );
    f.render_widget(list, area);
}

fn render_managed_versions(f: &mut Frame, area: Rect, versions: &HashMap<String, String>) {
    let mut sorted_versions: Vec<(&String, &String)> = versions.iter().collect();
    sorted_versions.sort_by(|a, b| a.0.cmp(b.0));

    let items: Vec<ListItem> = if sorted_versions.is_empty() {
        vec![ListItem::new("No managed versions")]
    } else {
        sorted_versions
            .iter()
            .map(|(tool, version)| {
                let line = Line::from(vec![
                    Span::styled(tool.as_str(), Style::default().fg(Color::Green)),
                    Span::raw(" → "),
                    Span::styled(version.as_str(), Style::default().fg(Color::White)),
                ]);
                ListItem::new(line)
            })
            .collect()
    };

    let list = List::new(items).block(
        Block::default()
            .title("Managed Versions")
            .borders(Borders::ALL),
    );
    f.render_widget(list, area);
}

fn render_missing_installs(f: &mut Frame, area: Rect, missing: &[String]) {
    let items: Vec<ListItem> = if missing.is_empty() {
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

    let list = List::new(items).block(
        Block::default()
            .title("Missing Installations")
            .borders(Borders::ALL),
    );
    f.render_widget(list, area);
}

fn render_warnings(f: &mut Frame, area: Rect, warnings: &[String]) {
    let items: Vec<ListItem> = if warnings.is_empty() {
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

    let list = List::new(items).block(
        Block::default()
            .title("Health Warnings")
            .borders(Borders::ALL),
    );
    f.render_widget(list, area);
}

fn render_disk_usage(f: &mut Frame, area: Rect, disk_usage: &Option<DiskUsage>) {
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
    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new("Press 'q' or ESC to exit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_tool_versions_empty() {
        use std::path::PathBuf;
        let path = PathBuf::from("/nonexistent/path");
        let versions = read_tool_versions(&path);
        assert!(versions.is_empty());
    }

    #[test]
    fn test_read_tool_versions_with_content() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "node 20.11.0").unwrap();
        writeln!(file, "go 1.23.5").unwrap();
        writeln!(file, "# comment").unwrap();
        writeln!(file, "").unwrap();
        file.flush().unwrap();

        let versions = read_tool_versions(file.path());
        assert_eq!(versions.len(), 2);
        assert_eq!(versions.get("node"), Some(&"20.11.0".to_string()));
        assert_eq!(versions.get("go"), Some(&"1.23.5".to_string()));
    }

    #[test]
    fn test_dir_size_empty_dir() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let size = dir_size(temp_dir.path()).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_dir_size_with_file() {
        use tempfile::TempDir;
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let size = dir_size(temp_dir.path()).unwrap();
        assert!(size > 0);
    }
}
