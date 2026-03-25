//! Installation lock module
//!
//! File-based exclusive lock to prevent multiple vex processes from installing the same tool version simultaneously.
//! Uses RAII pattern, lock is automatically released and lock file cleaned up when [`InstallLock`] is destroyed.
//! Includes stale lock detection via PID liveness check.

use crate::error::{Result, VexError};
use fs2::FileExt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
#[cfg(test)]
mod tests;

/// RAII-style installation exclusive lock
///
/// Uses non-blocking exclusive `flock`, lock file located at `~/.vex/locks/<tool>-<version>.lock`.
/// Automatically releases lock and deletes lock file on `Drop`.
pub struct InstallLock {
    file: File,
    path: PathBuf,
}

impl InstallLock {
    /// Acquire exclusive lock for specified tool version (non-blocking)
    ///
    /// # Arguments
    /// - `vex_dir` - vex root directory (`~/.vex`)
    /// - `tool` - Tool name
    /// - `version` - Version number
    ///
    /// # Errors
    /// - `VexError::LockConflict` - Lock already held by another process
    pub fn acquire(vex_dir: &Path, tool: &str, version: &str) -> Result<Self> {
        let locks_dir = vex_dir.join("locks");
        fs::create_dir_all(&locks_dir)?;

        let lock_filename = format!("{}-{}.lock", tool, version);
        let lock_path = locks_dir.join(lock_filename);

        // Check for stale lock before attempting to acquire
        if lock_path.exists() {
            if let Ok(mut file) = File::open(&lock_path) {
                let mut pid_str = String::new();
                if file.read_to_string(&mut pid_str).is_ok() {
                    if let Ok(pid) = pid_str.trim().parse::<i32>() {
                        // Check if process is still alive using kill(pid, 0)
                        #[cfg(unix)]
                        {
                            let is_alive = unsafe { libc::kill(pid, 0) } == 0;
                            if !is_alive {
                                // Stale lock - remove it
                                let _ = fs::remove_file(&lock_path);
                            }
                        }
                    }
                }
            }
        }

        let mut file = File::create(&lock_path)?;

        // Non-blocking exclusive lock
        if file.try_lock_exclusive().is_err() {
            return Err(VexError::LockConflict {
                tool: tool.to_string(),
                version: version.to_string(),
            });
        }

        // Write current PID to lock file
        let pid = std::process::id();
        write!(file, "{}", pid)?;
        file.flush()?;

        Ok(Self {
            file,
            path: lock_path,
        })
    }
}

impl Drop for InstallLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
        let _ = fs::remove_file(&self.path);
    }
}
