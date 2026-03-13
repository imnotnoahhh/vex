use super::types::{push_check, CheckStatus, DoctorCheck, DoctorReport};
use crate::config;
use crate::error::{Result, VexError};
use crate::project;
use crate::resolver;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub(super) fn collect() -> Result<DoctorReport> {
    use std::os::unix::fs::PermissionsExt;

    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let mut issues = 0;
    let mut warnings = 0;
    let mut checks = Vec::new();

    let directory_exists = vex_dir.exists();
    push_check(
        &mut checks,
        "vex_directory",
        if directory_exists {
            CheckStatus::Ok
        } else {
            issues += 1;
            CheckStatus::Error
        },
        if directory_exists {
            "vex directory exists"
        } else {
            "vex directory is missing"
        },
        if directory_exists {
            Vec::new()
        } else {
            vec!["Run 'vex init' to initialize the directory structure".to_string()]
        },
    );

    let required_dirs = ["cache", "locks", "toolchains", "current", "bin"];
    let missing_dirs = required_dirs
        .iter()
        .filter(|dir| !vex_dir.join(dir).exists())
        .map(|dir| dir.to_string())
        .collect::<Vec<_>>();
    push_check(
        &mut checks,
        "directory_structure",
        if missing_dirs.is_empty() {
            CheckStatus::Ok
        } else {
            issues += 1;
            CheckStatus::Error
        },
        if missing_dirs.is_empty() {
            "required vex directories exist"
        } else {
            "required vex directories are missing"
        },
        if missing_dirs.is_empty() {
            Vec::new()
        } else {
            let mut details = missing_dirs
                .iter()
                .map(|dir| format!("Missing: {}", dir))
                .collect::<Vec<_>>();
            details.push("Run 'vex init' to restore the missing directories".to_string());
            details
        },
    );

    let vex_bin = config::bin_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    let path_check = match std::env::var("PATH") {
        Ok(path_var) if path_var.contains(&vex_bin.to_string_lossy().to_string()) => DoctorCheck {
            id: "path".to_string(),
            status: CheckStatus::Ok,
            summary: "vex/bin is present in PATH".to_string(),
            details: Vec::new(),
        },
        Ok(_) => {
            warnings += 1;
            DoctorCheck {
                id: "path".to_string(),
                status: CheckStatus::Warn,
                summary: "vex/bin is not present in PATH".to_string(),
                details: vec![
                    "Add 'export PATH=\"$HOME/.vex/bin:$PATH\"' to your shell config".to_string(),
                ],
            }
        }
        Err(_) => {
            issues += 1;
            DoctorCheck {
                id: "path".to_string(),
                status: CheckStatus::Error,
                summary: "PATH is not set".to_string(),
                details: vec!["Your shell environment is missing PATH".to_string()],
            }
        }
    };
    checks.push(path_check);

    let path_priority_check = collect_path_priority_check(&vex_bin);
    if path_priority_check.status == CheckStatus::Warn {
        warnings += 1;
    }
    checks.push(path_priority_check);

    let shell = std::env::var("SHELL").unwrap_or_default();
    let shell_check = collect_shell_hook_check(&shell);
    if shell_check.status == CheckStatus::Warn {
        warnings += 1;
    }
    checks.push(shell_check);

    let config_check = collect_config_check(&vex_dir);
    if config_check.status == CheckStatus::Warn {
        warnings += 1;
    } else if config_check.status == CheckStatus::Error {
        issues += 1;
    }
    checks.push(config_check);

    let global_versions_check = collect_tool_versions_file_check(&vex_dir.join("tool-versions"));
    if global_versions_check.status == CheckStatus::Warn {
        warnings += 1;
    }
    checks.push(global_versions_check);

    let project_config_check = collect_project_config_check();
    if project_config_check.status == CheckStatus::Warn {
        warnings += 1;
    }
    checks.push(project_config_check);

    let effective_settings_check = collect_effective_settings_check();
    if effective_settings_check.status == CheckStatus::Warn {
        warnings += 1;
    }
    checks.push(effective_settings_check);

    let duplicate_hook_check = collect_duplicate_hook_check(&shell);
    if duplicate_hook_check.status == CheckStatus::Warn {
        warnings += 1;
    }
    checks.push(duplicate_hook_check);

    let toolchains_dir = config::toolchains_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    let tool_count = if toolchains_dir.exists() {
        fs::read_dir(&toolchains_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry
                            .file_type()
                            .ok()
                            .map(|ft| ft.is_dir())
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };
    push_check(
        &mut checks,
        "installed_tools",
        if toolchains_dir.exists() && tool_count > 0 {
            CheckStatus::Ok
        } else if toolchains_dir.exists() {
            warnings += 1;
            CheckStatus::Warn
        } else {
            issues += 1;
            CheckStatus::Error
        },
        if toolchains_dir.exists() && tool_count > 0 {
            "installed toolchains found"
        } else if toolchains_dir.exists() {
            "no tools are installed yet"
        } else {
            "toolchains directory is missing"
        },
        if toolchains_dir.exists() && tool_count > 0 {
            vec![format!("Installed tools: {}", tool_count)]
        } else if toolchains_dir.exists() {
            vec!["Run 'vex install <tool>' to install your first tool".to_string()]
        } else {
            vec!["Run 'vex init' to restore the toolchains directory".to_string()]
        },
    );

    let (broken_links, corepack_missing) = collect_broken_links(&vex_dir);
    let symlink_check = if broken_links.is_empty() {
        DoctorCheck {
            id: "symlinks".to_string(),
            status: CheckStatus::Ok,
            summary: "active symlinks are valid".to_string(),
            details: if corepack_missing {
                vec!["Corepack is not bundled with Node.js 25+, which is expected".to_string()]
            } else {
                Vec::new()
            },
        }
    } else {
        warnings += 1;
        DoctorCheck {
            id: "symlinks".to_string(),
            status: CheckStatus::Warn,
            summary: "broken symlinks were found".to_string(),
            details: broken_links,
        }
    };
    checks.push(symlink_check);

    let non_executable = fs::read_dir(&vex_bin)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            if metadata.is_symlink() || (metadata.permissions().mode() & 0o111) != 0 {
                return None;
            }
            Some(entry.file_name().to_string_lossy().to_string())
        })
        .collect::<Vec<_>>();
    push_check(
        &mut checks,
        "binary_permissions",
        if non_executable.is_empty() {
            CheckStatus::Ok
        } else {
            warnings += 1;
            CheckStatus::Warn
        },
        if non_executable.is_empty() {
            "vex-managed binaries are executable"
        } else {
            "some vex-managed binaries are not executable"
        },
        non_executable,
    );

    let failed_binaries = collect_failed_binaries(&vex_bin);
    push_check(
        &mut checks,
        "binary_runnability",
        if failed_binaries.is_empty() {
            CheckStatus::Ok
        } else {
            warnings += 1;
            CheckStatus::Warn
        },
        if failed_binaries.is_empty() {
            "managed binaries respond to probe commands"
        } else {
            "some binaries did not respond to probe commands"
        },
        failed_binaries,
    );

    let cache_check = collect_cache_integrity_check(&vex_dir);
    if cache_check.status == CheckStatus::Warn {
        warnings += 1;
    }
    checks.push(cache_check);

    let network_check = match Command::new("ping")
        .args(["-c", "1", "-W", "2", "nodejs.org"])
        .output()
    {
        Ok(output) if output.status.success() => DoctorCheck {
            id: "network".to_string(),
            status: CheckStatus::Ok,
            summary: "basic network connectivity is available".to_string(),
            details: Vec::new(),
        },
        _ => {
            warnings += 1;
            DoctorCheck {
                id: "network".to_string(),
                status: CheckStatus::Warn,
                summary: "nodejs.org was unreachable during the health check".to_string(),
                details: vec!["Check your internet connection or firewall settings".to_string()],
            }
        }
    };
    checks.push(network_check);

    Ok(DoctorReport {
        root: vex_dir.display().to_string(),
        issues,
        warnings,
        checks,
    })
}

fn collect_shell_hook_check(shell: &str) -> DoctorCheck {
    if shell.contains("zsh") {
        return shell_hook_check("zsh", ".zshrc", "vex env zsh", "eval \"$(vex env zsh)\"");
    }
    if shell.contains("bash") {
        return shell_hook_check(
            "bash",
            ".bashrc",
            "vex env bash",
            "eval \"$(vex env bash)\"",
        );
    }

    DoctorCheck {
        id: "shell_hook".to_string(),
        status: CheckStatus::Warn,
        summary: "unable to determine the active shell hook status".to_string(),
        details: vec!["The current shell is not zsh or bash".to_string()],
    }
}

fn shell_hook_check(
    shell_name: &str,
    file_name: &str,
    marker: &str,
    suggested: &str,
) -> DoctorCheck {
    let home = std::env::var("HOME").unwrap_or_default();
    let shell_rc = PathBuf::from(home).join(file_name);

    if !shell_rc.exists() {
        return DoctorCheck {
            id: "shell_hook".to_string(),
            status: CheckStatus::Warn,
            summary: format!("{} shell config file was not found", shell_name),
            details: vec![format!(
                "Create {} and add {}",
                shell_rc.display(),
                suggested
            )],
        };
    }

    match fs::read_to_string(&shell_rc) {
        Ok(content) if content.contains(marker) => DoctorCheck {
            id: "shell_hook".to_string(),
            status: CheckStatus::Ok,
            summary: format!("{} shell hook is configured", shell_name),
            details: Vec::new(),
        },
        Ok(_) => DoctorCheck {
            id: "shell_hook".to_string(),
            status: CheckStatus::Warn,
            summary: format!("{} shell hook is not configured", shell_name),
            details: vec![format!("Add {} to {}", suggested, shell_rc.display())],
        },
        Err(_) => DoctorCheck {
            id: "shell_hook".to_string(),
            status: CheckStatus::Warn,
            summary: format!("{} shell config could not be read", shell_name),
            details: vec![format!("Check permissions for {}", shell_rc.display())],
        },
    }
}

fn collect_config_check(vex_dir: &std::path::Path) -> DoctorCheck {
    let config_path = vex_dir.join("config.toml");
    if !config_path.exists() {
        return DoctorCheck {
            id: "config".to_string(),
            status: CheckStatus::Warn,
            summary: "config.toml is missing".to_string(),
            details: vec!["Run 'vex init' to recreate ~/.vex/config.toml".to_string()],
        };
    }

    match crate::config::load_settings_from_file(&config_path) {
        Ok(_) => DoctorCheck {
            id: "config".to_string(),
            status: CheckStatus::Ok,
            summary: "config.toml is valid".to_string(),
            details: Vec::new(),
        },
        Err(err) => DoctorCheck {
            id: "config".to_string(),
            status: CheckStatus::Warn,
            summary: "config.toml could not be parsed".to_string(),
            details: vec![err.to_string()],
        },
    }
}

fn collect_duplicate_hook_check(shell: &str) -> DoctorCheck {
    let Some((shell_name, file_name, marker)) = shell_hook_target(shell) else {
        return DoctorCheck {
            id: "shell_hook_duplicates".to_string(),
            status: CheckStatus::Warn,
            summary: "unable to check shell hook duplication for this shell".to_string(),
            details: vec!["The current shell is not zsh or bash".to_string()],
        };
    };

    let home = std::env::var("HOME").unwrap_or_default();
    let shell_rc = PathBuf::from(home).join(file_name);
    let content = match fs::read_to_string(&shell_rc) {
        Ok(content) => content,
        Err(_) => {
            return DoctorCheck {
                id: "shell_hook_duplicates".to_string(),
                status: CheckStatus::Ok,
                summary: format!("{} shell hook duplication could not be checked", shell_name),
                details: Vec::new(),
            };
        }
    };

    let count = content.matches(marker).count();
    if count > 1 {
        DoctorCheck {
            id: "shell_hook_duplicates".to_string(),
            status: CheckStatus::Warn,
            summary: format!("{} shell hook appears multiple times", shell_name),
            details: vec![format!("Found {} occurrences of '{}'", count, marker)],
        }
    } else {
        DoctorCheck {
            id: "shell_hook_duplicates".to_string(),
            status: CheckStatus::Ok,
            summary: format!("{} shell hook appears once", shell_name),
            details: Vec::new(),
        }
    }
}

fn shell_hook_target(shell: &str) -> Option<(&'static str, &'static str, &'static str)> {
    if shell.contains("zsh") {
        return Some(("zsh", ".zshrc", "vex env zsh"));
    }
    if shell.contains("bash") {
        return Some(("bash", ".bashrc", "vex env bash"));
    }
    None
}

fn collect_path_priority_check(vex_bin: &std::path::Path) -> DoctorCheck {
    let Ok(path_var) = std::env::var("PATH") else {
        return DoctorCheck {
            id: "path_priority".to_string(),
            status: CheckStatus::Warn,
            summary: "PATH priority could not be inspected".to_string(),
            details: vec!["PATH is not set".to_string()],
        };
    };

    let path_entries = path_var.split(':').collect::<Vec<_>>();
    let vex_bin_str = vex_bin.to_string_lossy().to_string();
    let Some(index) = path_entries.iter().position(|entry| *entry == vex_bin_str) else {
        return DoctorCheck {
            id: "path_priority".to_string(),
            status: CheckStatus::Warn,
            summary: "vex/bin is not present early enough in PATH".to_string(),
            details: vec![
                "Add ~/.vex/bin near the front of PATH to avoid binary conflicts".to_string(),
            ],
        };
    };

    if index == 0 {
        DoctorCheck {
            id: "path_priority".to_string(),
            status: CheckStatus::Ok,
            summary: "vex/bin has first PATH priority".to_string(),
            details: Vec::new(),
        }
    } else {
        DoctorCheck {
            id: "path_priority".to_string(),
            status: CheckStatus::Warn,
            summary: "vex/bin is present in PATH but not first".to_string(),
            details: vec![format!(
                "~/.vex/bin appears at PATH position {}. Earlier entries may shadow managed binaries.",
                index + 1
            )],
        }
    }
}

fn collect_broken_links(vex_dir: &std::path::Path) -> (Vec<String>, bool) {
    let current_dir = vex_dir.join("current");
    let bin_dir = vex_dir.join("bin");
    let mut broken_links = Vec::new();
    let mut corepack_missing = false;

    if current_dir.exists() {
        if let Ok(entries) = fs::read_dir(&current_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if fs::read_link(entry.path()).is_ok() && entry.path().canonicalize().is_err() {
                    broken_links.push(format!("current/{}", entry.file_name().to_string_lossy()));
                }
            }
        }
    }

    if bin_dir.exists() {
        if let Ok(entries) = fs::read_dir(&bin_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let filename = entry.file_name().to_string_lossy().to_string();
                if entry.path().canonicalize().is_err() && fs::read_link(entry.path()).is_ok() {
                    if filename == "corepack" {
                        corepack_missing = true;
                    } else {
                        broken_links.push(format!("bin/{}", filename));
                    }
                }
            }
        }
    }

    (broken_links, corepack_missing)
}

fn collect_failed_binaries(bin_dir: &std::path::Path) -> Vec<String> {
    let mut failed = Vec::new();
    if !bin_dir.exists() {
        return failed;
    }

    if let Ok(entries) = fs::read_dir(bin_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let bin_path = entry.path();
            let bin_name = entry.file_name().to_string_lossy().to_string();

            if should_skip_binary_probe(&bin_name) {
                continue;
            }

            let test_commands: Vec<Vec<&str>> = if bin_name.starts_with("go") {
                vec![vec!["version"], vec!["--version"], vec!["--help"]]
            } else if bin_name.starts_with('j') && bin_name.len() > 1 {
                vec![vec!["-version"], vec!["--version"], vec!["--help"]]
            } else {
                vec![vec!["--version"], vec!["--help"], vec!["-V"]]
            };

            let mut success = false;
            for args in test_commands {
                match Command::new(&bin_path)
                    .args(&args)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                {
                    Ok(mut child) => {
                        let timeout = std::time::Duration::from_secs(2);
                        let start = std::time::Instant::now();
                        loop {
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    if status.success() {
                                        if let Ok(output) = child.wait_with_output() {
                                            if !output.stdout.is_empty()
                                                || !output.stderr.is_empty()
                                            {
                                                success = true;
                                            }
                                        }
                                    }
                                    break;
                                }
                                Ok(None) => {
                                    if start.elapsed() > timeout {
                                        let _ = child.kill();
                                        break;
                                    }
                                    std::thread::sleep(std::time::Duration::from_millis(50));
                                }
                                Err(_) => break,
                            }
                        }

                        if success {
                            break;
                        }
                    }
                    Err(_) => continue,
                }
            }

            if !success {
                failed.push(bin_name);
            }
        }
    }

    failed
}

fn collect_cache_integrity_check(vex_dir: &std::path::Path) -> DoctorCheck {
    let cache_dir = vex_dir.join("cache");
    if !cache_dir.exists() {
        return DoctorCheck {
            id: "cache_integrity".to_string(),
            status: CheckStatus::Ok,
            summary: "cache directory is absent".to_string(),
            details: Vec::new(),
        };
    }

    let mut invalid_files = Vec::new();
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !(name.starts_with("remote-") && name.ends_with(".json")) {
                continue;
            }

            match fs::read_to_string(&path) {
                Ok(content) if serde_json::from_str::<serde_json::Value>(&content).is_ok() => {}
                Ok(_) => invalid_files.push(path.display().to_string()),
                Err(err) => invalid_files.push(format!("{} ({})", path.display(), err)),
            }
        }
    }

    if invalid_files.is_empty() {
        DoctorCheck {
            id: "cache_integrity".to_string(),
            status: CheckStatus::Ok,
            summary: "remote version cache files are readable".to_string(),
            details: Vec::new(),
        }
    } else {
        DoctorCheck {
            id: "cache_integrity".to_string(),
            status: CheckStatus::Warn,
            summary: "some remote cache files are invalid".to_string(),
            details: invalid_files,
        }
    }
}

fn collect_tool_versions_file_check(path: &std::path::Path) -> DoctorCheck {
    if !path.exists() {
        return DoctorCheck {
            id: "global_tool_versions".to_string(),
            status: CheckStatus::Ok,
            summary: "global tool-versions file is not set".to_string(),
            details: Vec::new(),
        };
    }

    let Ok(content) = fs::read_to_string(path) else {
        return DoctorCheck {
            id: "global_tool_versions".to_string(),
            status: CheckStatus::Warn,
            summary: "global tool-versions file could not be read".to_string(),
            details: vec![path.display().to_string()],
        };
    };

    let mut invalid_lines = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 2 {
            invalid_lines.push(format!("Line {}: {}", index + 1, line));
        }
    }

    if invalid_lines.is_empty() {
        DoctorCheck {
            id: "global_tool_versions".to_string(),
            status: CheckStatus::Ok,
            summary: "global tool-versions file is valid".to_string(),
            details: Vec::new(),
        }
    } else {
        DoctorCheck {
            id: "global_tool_versions".to_string(),
            status: CheckStatus::Warn,
            summary: "global tool-versions file contains invalid lines".to_string(),
            details: invalid_lines,
        }
    }
}

fn collect_project_config_check() -> DoctorCheck {
    let cwd = resolver::current_dir();
    match project::load_nearest_project_config(&cwd) {
        Ok(Some(loaded)) => {
            let mut details = vec![format!("Project config: {}", loaded.path.display())];
            details.push(format!("Command tasks: {}", loaded.config.commands.len()));
            details.push(format!("Project env vars: {}", loaded.config.env.len()));
            details.push(format!("Project mirrors: {}", loaded.config.mirrors.len()));
            details.push(format!(
                "Project network overrides: {}",
                count_project_network_overrides(&loaded.config.network)
            ));
            if let Some(auto_switch) = loaded.config.behavior.auto_switch {
                details.push(format!("auto_switch = {}", auto_switch));
            }
            if let Some(auto_activate_venv) = loaded.config.behavior.auto_activate_venv {
                details.push(format!("auto_activate_venv = {}", auto_activate_venv));
            }
            if let Some(non_interactive) = loaded.config.behavior.non_interactive {
                details.push(format!("non_interactive = {}", non_interactive));
            }

            DoctorCheck {
                id: "project_config".to_string(),
                status: CheckStatus::Ok,
                summary: "nearest .vex.toml is valid".to_string(),
                details,
            }
        }
        Ok(None) => DoctorCheck {
            id: "project_config".to_string(),
            status: CheckStatus::Ok,
            summary: "no .vex.toml found in the current project tree".to_string(),
            details: Vec::new(),
        },
        Err(err) => DoctorCheck {
            id: "project_config".to_string(),
            status: CheckStatus::Warn,
            summary: ".vex.toml could not be parsed".to_string(),
            details: vec![err.to_string()],
        },
    }
}

fn collect_effective_settings_check() -> DoctorCheck {
    let cwd = resolver::current_dir();
    let settings = match config::load_effective_settings(&cwd) {
        Ok(settings) => settings,
        Err(err) => {
            return DoctorCheck {
                id: "effective_settings".to_string(),
                status: CheckStatus::Warn,
                summary: "effective configuration could not be loaded".to_string(),
                details: vec![err.to_string()],
            };
        }
    };
    let mut details = vec![
        format!(
            "connect_timeout = {}s",
            settings.network.connect_timeout.as_secs()
        ),
        format!(
            "read_timeout = {}s",
            settings.network.read_timeout.as_secs()
        ),
        format!("download_retries = {}", settings.network.download_retries),
        format!(
            "max_concurrent_downloads = {}",
            settings.network.max_concurrent_downloads
        ),
        format!(
            "max_http_redirects = {}",
            settings.network.max_http_redirects
        ),
        format!(
            "proxy = {}",
            settings
                .network
                .proxy
                .clone()
                .unwrap_or_else(|| "(not set)".to_string())
        ),
        format!("mirror_count = {}", settings.mirrors.len()),
        format!("auto_switch = {}", settings.behavior.auto_switch),
        format!(
            "auto_activate_venv = {}",
            settings.behavior.auto_activate_venv
        ),
        format!("non_interactive = {}", settings.behavior.non_interactive),
    ];

    let mut invalid = Vec::new();
    if let Some(proxy) = &settings.network.proxy {
        if let Err(err) = reqwest::Proxy::all(proxy) {
            invalid.push(format!("Invalid proxy URL: {} ({})", proxy, err));
        }
    }

    for (tool, mirror) in &settings.mirrors {
        if let Err(err) = reqwest::Url::parse(mirror) {
            invalid.push(format!("Invalid mirror for {}: {} ({})", tool, mirror, err));
        }
    }

    if invalid.is_empty() {
        DoctorCheck {
            id: "effective_settings".to_string(),
            status: CheckStatus::Ok,
            summary: "effective configuration is valid".to_string(),
            details,
        }
    } else {
        details.extend(invalid);
        DoctorCheck {
            id: "effective_settings".to_string(),
            status: CheckStatus::Warn,
            summary: "effective configuration contains invalid values".to_string(),
            details,
        }
    }
}

fn count_project_network_overrides(network: &project::ProjectNetworkConfig) -> usize {
    [
        network.connect_timeout_secs.is_some(),
        network.read_timeout_secs.is_some(),
        network.download_retries.is_some(),
        network.retry_base_delay_secs.is_some(),
        network.max_concurrent_downloads.is_some(),
        network.max_http_redirects.is_some(),
        network.proxy.is_some(),
    ]
    .into_iter()
    .filter(|value| *value)
    .count()
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
