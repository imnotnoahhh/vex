//! Tool installation module
//!
//! Responsible for downloading, verifying, extracting, and installing tool versions to `~/.vex/toolchains/`.
//! Includes disk space checking, path traversal protection, and `CleanupGuard` automatic cleanup mechanism.
//!
//! # Features
//!
//! - **Parallel extraction**: Files are extracted in parallel using rayon (directories created sequentially)
//! - **Path safety**: All archive paths are validated to prevent path traversal attacks
//! - **Atomic operations**: Installation uses temporary directories and atomic moves
//! - **Automatic cleanup**: Failed installations automatically clean up temporary files

mod extract;
mod offline;
mod online;
mod support;
#[cfg(test)]
mod tests;

use crate::error::Result;
use crate::tools::Tool;

/// Install specified tool version with offline mode support
pub fn install_with_mode(tool: &dyn Tool, version: &str, offline: bool) -> Result<()> {
    if offline {
        offline::install_offline(tool, version)
    } else {
        online::install(tool, version)
    }
}

/// Install specified tool version
pub fn install(tool: &dyn Tool, version: &str) -> Result<()> {
    online::install(tool, version)
}
