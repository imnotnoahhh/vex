//! Terminal UI rendering primitives
//!
//! Provides shared rendering components for consistent terminal output across all commands.
//! Supports both interactive and non-interactive modes, with JSON output compatibility.
//!
//! # Components
//!
//! - **Headers**: Section titles and command headers
//! - **Status**: Success/warning/error indicators
//! - **Tables**: Aligned rows and columns
//! - **Progress**: Spinners and progress bars
//! - **Summaries**: Final status summaries
//! - **Prompts**: Interactive user prompts

use indicatif::{ProgressBar as IndicatifProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::io;
use std::time::Duration;

/// UI context for rendering
#[derive(Debug, Clone)]
pub struct UiContext {
    /// Whether to use interactive features (spinners, progress bars)
    pub interactive: bool,
}

impl UiContext {
    /// Create a new UI context
    pub fn new() -> Self {
        Self {
            interactive: atty::is(atty::Stream::Stdout),
        }
    }

    /// Create a non-interactive context (for testing or piped output)
    #[allow(dead_code)]
    pub fn non_interactive() -> Self {
        Self { interactive: false }
    }
}

impl Default for UiContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a section header
pub fn header(text: &str) {
    println!();
    println!("{}", text.bold());
    println!();
}

/// Render a success message
pub fn success(text: &str) {
    println!("{} {}", "✓".green(), text);
}

/// Render a warning message
pub fn warning(text: &str) {
    println!("{} {}", "⚠".yellow(), text);
}

/// Render an error message
pub fn error(text: &str) {
    println!("{} {}", "✗".red(), text);
}

/// Render an info message
pub fn info(text: &str) {
    println!("{} {}", "→".cyan(), text);
}

/// Render a dimmed/secondary message
pub fn dimmed(text: &str) {
    println!("{}", text.dimmed());
}

/// Table builder for aligned output
#[derive(Debug)]
pub struct Table {
    rows: Vec<Vec<String>>,
    headers: Option<Vec<String>>,
}

impl Table {
    /// Create a new table
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            headers: None,
        }
    }

    /// Set table headers
    #[allow(dead_code)]
    pub fn headers(mut self, headers: Vec<String>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Add a row to the table
    pub fn row(mut self, row: Vec<String>) -> Self {
        self.rows.push(row);
        self
    }

    /// Render the table
    pub fn render(&self) {
        if self.rows.is_empty() && self.headers.is_none() {
            return;
        }

        // Calculate column widths
        let mut col_widths = Vec::new();

        if let Some(headers) = &self.headers {
            col_widths = headers.iter().map(|h| h.len()).collect();
        }

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i >= col_widths.len() {
                    col_widths.push(cell.len());
                } else {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }

        // Render headers
        if let Some(headers) = &self.headers {
            print!("  ");
            for (i, header) in headers.iter().enumerate() {
                print!("{:<width$}", header.bold(), width = col_widths[i]);
                if i < headers.len() - 1 {
                    print!("  ");
                }
            }
            println!();
            println!();
        }

        // Render rows
        for row in &self.rows {
            print!("  ");
            for (i, cell) in row.iter().enumerate() {
                let width = col_widths.get(i).copied().unwrap_or(0);
                print!("{:<width$}", cell, width = width);
                if i < row.len() - 1 {
                    print!("  ");
                }
            }
            println!();
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress indicator for long-running operations
#[allow(dead_code)]
pub struct Progress {
    bar: Option<IndicatifProgressBar>,
    ctx: UiContext,
}

#[allow(dead_code)]
impl Progress {
    /// Create a new progress indicator with a message
    pub fn new(ctx: &UiContext, message: &str) -> Self {
        let bar = if ctx.interactive {
            let pb = IndicatifProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .unwrap()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
            pb.set_message(message.to_string());
            pb.enable_steady_tick(Duration::from_millis(80));
            Some(pb)
        } else {
            println!("{} {}...", "→".cyan(), message);
            None
        };

        Self {
            bar,
            ctx: ctx.clone(),
        }
    }

    /// Update the progress message
    pub fn set_message(&self, message: &str) {
        if let Some(bar) = &self.bar {
            bar.set_message(message.to_string());
        } else if !self.ctx.interactive {
            println!("{} {}...", "→".cyan(), message);
        }
    }

    /// Finish with a success message
    pub fn finish_with_success(self, message: &str) {
        if let Some(bar) = self.bar {
            bar.finish_with_message(format!("{} {}", "✓".green(), message));
        } else {
            success(message);
        }
    }

    /// Finish with an error message
    pub fn finish_with_error(self, message: &str) {
        if let Some(bar) = self.bar {
            bar.finish_with_message(format!("{} {}", "✗".red(), message));
        } else {
            error(message);
        }
    }

    /// Finish and clear the progress indicator
    pub fn finish_and_clear(self) {
        if let Some(bar) = self.bar {
            bar.finish_and_clear();
        }
    }
}

/// Progress bar for operations with known total
/// Progress bar for operations with known total
#[allow(dead_code)]
pub struct ProgressBar {
    bar: Option<IndicatifProgressBar>,
    ctx: UiContext,
}

#[allow(dead_code)]
impl ProgressBar {
    /// Create a new progress bar
    pub fn new(ctx: &UiContext, total: u64, message: &str) -> Self {
        let bar = if ctx.interactive {
            let pb = IndicatifProgressBar::new(total);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
                    .unwrap()
                    .progress_chars("█▓▒░ "),
            );
            pb.set_message(message.to_string());
            Some(pb)
        } else {
            println!("{} {}...", "→".cyan(), message);
            None
        };

        Self {
            bar,
            ctx: ctx.clone(),
        }
    }

    /// Increment the progress
    pub fn inc(&self, delta: u64) {
        if let Some(bar) = &self.bar {
            bar.inc(delta);
        }
    }

    /// Set the current position
    pub fn set_position(&self, pos: u64) {
        if let Some(bar) = &self.bar {
            bar.set_position(pos);
        }
    }

    /// Finish the progress bar
    pub fn finish(self) {
        if let Some(bar) = self.bar {
            bar.finish();
        }
    }

    /// Finish and clear the progress bar
    pub fn finish_and_clear(self) {
        if let Some(bar) = self.bar {
            bar.finish_and_clear();
        }
    }
}

/// Summary builder for final status output
#[derive(Debug)]
pub struct Summary {
    items: Vec<(SummaryStatus, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryStatus {
    Success,
    Warning,
    Error,
    Info,
}

impl Summary {
    /// Create a new summary
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a success item
    pub fn success(mut self, text: String) -> Self {
        self.items.push((SummaryStatus::Success, text));
        self
    }

    /// Add a warning item
    pub fn warning(mut self, text: String) -> Self {
        self.items.push((SummaryStatus::Warning, text));
        self
    }

    /// Add an error item
    pub fn error(mut self, text: String) -> Self {
        self.items.push((SummaryStatus::Error, text));
        self
    }

    /// Add an info item
    pub fn info(mut self, text: String) -> Self {
        self.items.push((SummaryStatus::Info, text));
        self
    }

    /// Render the summary
    pub fn render(&self) {
        if self.items.is_empty() {
            return;
        }

        println!();
        for (status, text) in &self.items {
            match status {
                SummaryStatus::Success => success(text),
                SummaryStatus::Warning => warning(text),
                SummaryStatus::Error => error(text),
                SummaryStatus::Info => info(text),
            }
        }
        println!();
    }
}

impl Default for Summary {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt for user confirmation
/// Prompt for confirmation (yes/no)
#[allow(dead_code)]
pub fn confirm(prompt: &str, default: bool) -> io::Result<bool> {
    use dialoguer::Confirm;

    Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()
        .map_err(io::Error::other)
}

/// Prompt for user selection from a list
#[allow(dead_code)]
pub fn select<T: ToString + Clone + std::fmt::Display>(
    prompt: &str,
    items: &[T],
) -> io::Result<usize> {
    use dialoguer::Select;

    Select::new()
        .with_prompt(prompt)
        .items(items)
        .interact()
        .map_err(io::Error::other)
}

/// Prompt for user input
#[allow(dead_code)]
pub fn input(prompt: &str) -> io::Result<String> {
    use dialoguer::Input;

    Input::new()
        .with_prompt(prompt)
        .interact_text()
        .map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_context_creation() {
        let ctx = UiContext::new();
        // Interactive status depends on terminal
        let _ = ctx.interactive;

        let ctx = UiContext::non_interactive();
        assert!(!ctx.interactive);
    }

    #[test]
    fn test_table_empty() {
        let table = Table::new();
        table.render(); // Should not panic
    }

    #[test]
    fn test_table_with_rows() {
        let table = Table::new()
            .headers(vec!["Tool".to_string(), "Version".to_string()])
            .row(vec!["node".to_string(), "20.0.0".to_string()])
            .row(vec!["go".to_string(), "1.21.0".to_string()]);

        table.render(); // Should not panic
    }

    #[test]
    fn test_summary_empty() {
        let summary = Summary::new();
        summary.render(); // Should not panic
    }

    #[test]
    fn test_summary_with_items() {
        let summary = Summary::new()
            .success("Installation complete".to_string())
            .warning("1 tool is outdated".to_string())
            .info("Run 'vex upgrade' to update".to_string());

        summary.render(); // Should not panic
    }

    #[test]
    fn test_progress_non_interactive() {
        let ctx = UiContext::non_interactive();
        let progress = Progress::new(&ctx, "Testing");
        progress.set_message("Still testing");
        progress.finish_with_success("Test complete");
    }

    #[test]
    fn test_progress_bar_non_interactive() {
        let ctx = UiContext::non_interactive();
        let bar = ProgressBar::new(&ctx, 100, "Processing");
        bar.inc(50);
        bar.set_position(75);
        bar.finish();
    }
}
