use super::primitives::{success, UiContext};
use indicatif::{ProgressBar as IndicatifProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::time::Duration;

/// Progress indicator for long-running operations
pub struct Progress {
    bar: Option<IndicatifProgressBar>,
    ctx: UiContext,
}

impl Progress {
    /// Create a new progress indicator with a message
    pub fn new(ctx: &UiContext, message: &str) -> Self {
        let bar = if ctx.interactive {
            let progress_bar = IndicatifProgressBar::new_spinner();
            progress_bar.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .unwrap()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
            progress_bar.set_message(message.to_string());
            progress_bar.enable_steady_tick(Duration::from_millis(80));
            Some(progress_bar)
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
            bar.finish_and_clear();
        }
        success(message);
    }
}
