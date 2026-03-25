use super::TEAM_CONFIG_FILE;
use crate::error::{Result, VexError};
use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const GIT_CLONE_TIMEOUT: Duration = Duration::from_secs(60);

pub(super) fn load_team_config_from_git_repo(source: &str, is_local: bool) -> Result<String> {
    let temp = TempDir::new()?;
    let clone_dir = temp.path().join("repo");
    let mut command = Command::new("git");
    if !is_local {
        command
            .env("GIT_TERMINAL_PROMPT", "0")
            .env(
                "GIT_SSH_COMMAND",
                "ssh -o BatchMode=yes -o ConnectTimeout=10",
            )
            .args(["-c", "http.lowSpeedLimit=1", "-c", "http.lowSpeedTime=30"]);
    }
    command
        .args(["clone", "--depth", "1", "--quiet", source])
        .arg(&clone_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let mut child = command.spawn()?;
    let deadline = Instant::now() + GIT_CLONE_TIMEOUT;
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output()?;
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VexError::Config(format!(
                "Timed out while cloning team config repository '{}' after {} seconds. {}",
                source,
                GIT_CLONE_TIMEOUT.as_secs(),
                stderr.trim()
            )));
        }
        thread::sleep(Duration::from_millis(100));
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(VexError::Config(format!(
            "Failed to clone team config repository '{}': {}",
            source,
            stderr.trim()
        )));
    }

    let config_path = clone_dir.join(TEAM_CONFIG_FILE);
    if !config_path.exists() {
        return Err(VexError::Config(format!(
            "Git repository '{}' does not contain {} at its root.",
            source, TEAM_CONFIG_FILE
        )));
    }

    fs::read_to_string(config_path).map_err(VexError::from)
}
