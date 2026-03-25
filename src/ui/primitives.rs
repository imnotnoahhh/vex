use owo_colors::OwoColorize;
use std::io::{self, IsTerminal};

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
            interactive: io::stdout().is_terminal(),
        }
    }

    /// Create a non-interactive context (for testing or piped output)
    #[cfg(test)]
    pub fn non_interactive() -> Self {
        Self { interactive: false }
    }
}

impl Default for UiContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn header(text: &str) {
    println!();
    println!("{}", text.bold());
    println!();
}

pub fn success(text: &str) {
    println!("{} {}", "✓".green(), text);
}

pub fn warning(text: &str) {
    println!("{} {}", "⚠".yellow(), text);
}

pub fn error(text: &str) {
    println!("{} {}", "✗".red(), text);
}

pub fn info(text: &str) {
    println!("{} {}", "→".cyan(), text);
}

pub fn dimmed(text: &str) {
    println!("{}", text.dimmed());
}
