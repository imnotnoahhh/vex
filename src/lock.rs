//! Installation lock module
//!
//! File-based exclusive lock to prevent multiple vex processes from installing the same tool version simultaneously.
//! Uses RAII pattern, lock is automatically released and lock file cleaned up when [`InstallLock`] is destroyed.

use crate::error::{Result, VexError};
use fs2::FileExt;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

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

        let file = File::create(&lock_path)?;

        // Non-blocking exclusive lock
        if file.try_lock_exclusive().is_err() {
            return Err(VexError::LockConflict {
                tool: tool.to_string(),
                version: version.to_string(),
            });
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Each test gets its own unique temp directory to avoid parallel interference.
    fn unique_vex_dir() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("vex-lock-test-{}-{}", std::process::id(), id));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_lock_acquire_success() {
        let vex_dir = unique_vex_dir();
        let lock = InstallLock::acquire(&vex_dir, "node", "20.11.0");
        assert!(lock.is_ok());
        let _ = fs::remove_dir_all(&vex_dir);
    }

    #[test]
    fn test_lock_file_created() {
        let vex_dir = unique_vex_dir();
        let lock_path = vex_dir.join("locks").join("go-1.23.5.lock");

        let _lock = InstallLock::acquire(&vex_dir, "go", "1.23.5").unwrap();
        assert!(lock_path.exists());

        let _ = fs::remove_dir_all(&vex_dir);
    }

    #[test]
    fn test_lock_cleanup_on_drop() {
        let vex_dir = unique_vex_dir();
        let lock_path = vex_dir.join("locks").join("node-18.0.0.lock");

        {
            let _lock = InstallLock::acquire(&vex_dir, "node", "18.0.0").unwrap();
            assert!(lock_path.exists());
        }

        // Lock file removed after drop
        assert!(!lock_path.exists());
        let _ = fs::remove_dir_all(&vex_dir);
    }

    #[test]
    fn test_lock_reacquire_after_drop() {
        let vex_dir = unique_vex_dir();

        {
            let _lock = InstallLock::acquire(&vex_dir, "rust", "1.93.1").unwrap();
        }

        // Should succeed after previous lock is dropped
        let lock2 = InstallLock::acquire(&vex_dir, "rust", "1.93.1");
        assert!(lock2.is_ok());
        let _ = fs::remove_dir_all(&vex_dir);
    }

    #[test]
    fn test_different_versions_no_conflict() {
        let vex_dir = unique_vex_dir();

        let _lock1 = InstallLock::acquire(&vex_dir, "node", "20.11.0").unwrap();
        let lock2 = InstallLock::acquire(&vex_dir, "node", "18.19.0");

        assert!(lock2.is_ok());
        let _ = fs::remove_dir_all(&vex_dir);
    }

    #[test]
    fn test_different_tools_no_conflict() {
        let vex_dir = unique_vex_dir();

        let _lock1 = InstallLock::acquire(&vex_dir, "node", "20.11.0").unwrap();
        let lock2 = InstallLock::acquire(&vex_dir, "go", "1.23.5");

        assert!(lock2.is_ok());
        let _ = fs::remove_dir_all(&vex_dir);
    }

    /// Cross-process lock conflict test.
    /// Spawns a child process that holds a lock, then verifies the parent
    /// cannot acquire the same lock.
    #[test]
    fn test_cross_process_lock_conflict() {
        let vex_dir = unique_vex_dir();
        let locks_dir = vex_dir.join("locks");
        fs::create_dir_all(&locks_dir).unwrap();

        let lock_path = locks_dir.join("node-22.0.0.lock");

        // Child process: Python script that acquires exclusive lock and signals readiness
        let python_script = format!(
            r#"
import fcntl
import sys
import time

with open('{}', 'w') as f:
    fcntl.flock(f.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    print('ready', flush=True)
    time.sleep(30)
"#,
            lock_path.display()
        );

        let mut child = std::process::Command::new("/usr/bin/python3")
            .arg("-c")
            .arg(&python_script)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("failed to spawn child");

        // Wait for child to signal it holds the lock
        use std::io::Read;
        let stdout = child.stdout.as_mut().unwrap();
        let mut buf = [0u8; 6];
        let mut total = 0;
        while total < 5 {
            let n = stdout.read(&mut buf[total..]).unwrap();
            if n == 0 {
                break;
            }
            total += n;
        }
        assert!(
            std::str::from_utf8(&buf[..total])
                .unwrap()
                .starts_with("ready"),
            "child did not acquire lock"
        );

        // Now try to acquire the same lock from this process - should fail
        let file = File::create(&lock_path).unwrap();
        let result = file.try_lock_exclusive();
        assert!(result.is_err(), "Expected lock conflict with child process");

        // Cleanup
        child.kill().ok();
        child.wait().ok();
        let _ = fs::remove_dir_all(&vex_dir);
    }

    #[test]
    fn test_locks_dir_auto_created() {
        let vex_dir = unique_vex_dir();
        let locks_dir = vex_dir.join("locks");

        // locks/ doesn't exist yet
        assert!(!locks_dir.exists());

        let _lock = InstallLock::acquire(&vex_dir, "java", "21").unwrap();
        assert!(locks_dir.exists());

        let _ = fs::remove_dir_all(&vex_dir);
    }
}
