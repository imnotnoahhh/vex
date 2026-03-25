use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub(super) fn collect_failed_binaries(bin_dir: &Path) -> Vec<String> {
    if !bin_dir.exists() {
        return Vec::new();
    }

    let Ok(entries) = fs::read_dir(bin_dir) else {
        return Vec::new();
    };

    entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let bin_name = entry.file_name().to_string_lossy().to_string();
            (!probe_succeeds(&entry.path(), &bin_name)).then_some(bin_name)
        })
        .collect()
}

fn probe_succeeds(bin_path: &Path, bin_name: &str) -> bool {
    if should_skip_binary_probe(bin_name) {
        return true;
    }

    candidate_commands(bin_name)
        .into_iter()
        .any(|args| run_probe_command(bin_path, &args))
}

fn candidate_commands(bin_name: &str) -> Vec<Vec<&'static str>> {
    if bin_name.starts_with("go") {
        vec![vec!["version"], vec!["--version"], vec!["--help"]]
    } else if bin_name.starts_with('j') && bin_name.len() > 1 {
        vec![vec!["-version"], vec!["--version"], vec!["--help"]]
    } else {
        vec![vec!["--version"], vec!["--help"], vec!["-V"]]
    }
}

fn run_probe_command(bin_path: &Path, args: &[&str]) -> bool {
    let Ok(mut child) = Command::new(bin_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    else {
        return false;
    };

    let timeout = Duration::from_secs(2);
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return false;
                }
                return child
                    .wait_with_output()
                    .map(|output| !output.stdout.is_empty() || !output.stderr.is_empty())
                    .unwrap_or(false);
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return false;
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return false,
        }
    }
}

fn should_skip_binary_probe(bin_name: &str) -> bool {
    bin_name.ends_with(".so")
        || bin_name.ends_with(".dylib")
        || bin_name.ends_with("-config")
        || bin_name.starts_with("idle")
        || bin_name == "corepack"
        || bin_name == "rust-gdb"
        || bin_name == "rust-lldb"
        || bin_name == "rmiregistry"
        || bin_name == "serialver"
        || bin_name == "jconsole"
        || bin_name == "jstatd"
}
