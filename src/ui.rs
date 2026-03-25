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

mod primitives;
mod progress;
mod summary;
mod table;
#[cfg(test)]
mod tests;

pub use primitives::{dimmed, error, header, info, success, warning, UiContext};
pub use progress::Progress;
pub use summary::Summary;
pub use table::Table;
