use super::super::types::{push_check, CheckStatus, DoctorCheck};
use super::system;
use crate::config::{self, StrictMode};
use crate::home_state::{self, AuditKind};
use crate::tools::python;
use crate::version_state;
use std::path::Path;

pub(super) fn collect_environment_checks(
    vex_dir: &Path,
    vex_bin: &Path,
    warnings: &mut usize,
    issues: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) {
    let directory_exists = vex_dir.exists();
    push_check(
        checks,
        "vex_directory",
        if directory_exists {
            CheckStatus::Ok
        } else {
            *issues += 1;
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
        checks,
        "directory_structure",
        if missing_dirs.is_empty() {
            CheckStatus::Ok
        } else {
            *issues += 1;
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

    let path_check = match std::env::var("PATH") {
        Ok(path_var) if path_var.contains(&vex_bin.to_string_lossy().to_string()) => DoctorCheck {
            id: "path".to_string(),
            status: CheckStatus::Ok,
            summary: "vex/bin is present in PATH".to_string(),
            details: Vec::new(),
        },
        Ok(_) => {
            *warnings += 1;
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
            *issues += 1;
            DoctorCheck {
                id: "path".to_string(),
                status: CheckStatus::Error,
                summary: "PATH is not set".to_string(),
                details: vec!["Your shell environment is missing PATH".to_string()],
            }
        }
    };
    checks.push(path_check);

    let path_priority_check = system::collect_path_priority_check(vex_bin);
    if path_priority_check.status == CheckStatus::Warn {
        *warnings += 1;
    }
    checks.push(path_priority_check);

    let tool_manager_conflict_check = system::collect_tool_manager_conflict_check(vex_bin);
    if tool_manager_conflict_check.status == CheckStatus::Warn {
        *warnings += 1;
    }
    checks.push(tool_manager_conflict_check);

    if vex_dir.join("toolchains/node").exists() || vex_dir.join("current/node").exists() {
        let npm_bin_dir = vex_dir.join("npm").join("prefix").join("bin");
        let npm_bin_str = npm_bin_dir.to_string_lossy().to_string();
        let npm_path_check = if !npm_bin_dir.exists() {
            *warnings += 1;
            DoctorCheck {
                id: "npm_global_bin_path".to_string(),
                status: CheckStatus::Warn,
                summary: "managed npm global bin directory is missing".to_string(),
                details: vec![
                    "Run 'vex init' to recreate ~/.vex/npm/prefix/bin, or reinstall Node with 'vex install node@<version> --force'".to_string(),
                ],
            }
        } else {
            match std::env::var("PATH") {
                Ok(path_var) if path_var.split(':').any(|entry| entry == npm_bin_str) => {
                    DoctorCheck {
                        id: "npm_global_bin_path".to_string(),
                        status: CheckStatus::Ok,
                        summary: "managed npm global bin dir is present in PATH".to_string(),
                        details: Vec::new(),
                    }
                }
                Ok(_) => {
                    *warnings += 1;
                    DoctorCheck {
                        id: "npm_global_bin_path".to_string(),
                        status: CheckStatus::Warn,
                        summary: "managed npm global bin dir is not present in PATH".to_string(),
                        details: vec![
                            "Re-run 'vex init --shell auto', or add ~/.vex/npm/prefix/bin before ~/.vex/bin in your shell PATH".to_string(),
                        ],
                    }
                }
                Err(_) => DoctorCheck {
                    id: "npm_global_bin_path".to_string(),
                    status: CheckStatus::Ok,
                    summary: "managed npm global bin dir could not be inspected".to_string(),
                    details: Vec::new(),
                },
            }
        };
        checks.push(npm_path_check);
    }

    let shell = std::env::var("SHELL").unwrap_or_default();
    let shell_check = system::collect_shell_hook_check(&shell);
    if shell_check.status == CheckStatus::Warn {
        *warnings += 1;
    }
    checks.push(shell_check);

    let config_check = system::collect_config_check(vex_dir);
    if config_check.status == CheckStatus::Warn {
        *warnings += 1;
    } else if config_check.status == CheckStatus::Error {
        *issues += 1;
    }
    checks.push(config_check);

    let global_versions_check =
        system::collect_tool_versions_file_check(&vex_dir.join("tool-versions"));
    if global_versions_check.status == CheckStatus::Warn {
        *warnings += 1;
    }
    checks.push(global_versions_check);

    let project_config_check = system::collect_project_config_check();
    if project_config_check.status == CheckStatus::Warn {
        *warnings += 1;
    }
    checks.push(project_config_check);

    let effective_settings_check = system::collect_effective_settings_check();
    if effective_settings_check.status == CheckStatus::Warn {
        *warnings += 1;
    }
    checks.push(effective_settings_check);

    let duplicate_hook_check = system::collect_duplicate_hook_check(&shell);
    if duplicate_hook_check.status == CheckStatus::Warn {
        *warnings += 1;
    }
    checks.push(duplicate_hook_check);

    collect_home_hygiene_check(warnings, issues, checks);
    collect_path_conflict_check(vex_bin, warnings, issues, checks);
    collect_captured_env_check(vex_dir, warnings, issues, checks);
    collect_python_base_check(vex_dir, warnings, checks);
    collect_manager_conflict_check(warnings, issues, checks);
}

fn collect_home_hygiene_check(
    warnings: &mut usize,
    issues: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) {
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return,
    };
    let audits = home_state::audit(&home, Some("all"));
    if audits.is_empty() {
        push_check(
            checks,
            "home_hygiene",
            CheckStatus::Ok,
            "supported home-directory state is already contained in ~/.vex",
            Vec::new(),
        );
        return;
    }

    let mode = config::strict_home_hygiene().unwrap_or(StrictMode::Warn);
    let status = strict_status(mode, warnings, issues);
    let mut details = audits
        .iter()
        .map(|audit| match audit.kind {
            AuditKind::SafeMigration => format!(
                "{} -> {}",
                audit.source.display(),
                audit
                    .destination
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "~/.vex".to_string())
            ),
            AuditKind::Advisory => format!("{} ({})", audit.source.display(), audit.summary),
        })
        .collect::<Vec<_>>();
    details.push("Run 'vex repair migrate-home' to preview safe migrations.".to_string());
    push_check(
        checks,
        "home_hygiene",
        status,
        "legacy home-directory state was found outside ~/.vex",
        details,
    );
}

fn collect_path_conflict_check(
    vex_bin: &Path,
    warnings: &mut usize,
    issues: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) {
    let path = std::env::var("PATH").unwrap_or_default();
    let vex_bin = vex_bin.to_string_lossy().to_string();
    let mut conflicts = Vec::new();
    for segment in path.split(':').filter(|segment| !segment.is_empty()) {
        if segment == vex_bin {
            break;
        }
        if segment.contains(".cargo/bin")
            || segment.ends_with("/go/bin")
            || segment.contains(".nvm")
            || segment.contains(".pyenv")
        {
            conflicts.push(segment.to_string());
        }
    }

    if conflicts.is_empty() {
        push_check(
            checks,
            "path_conflicts",
            CheckStatus::Ok,
            "PATH keeps ~/.vex entries ahead of common legacy manager bins",
            Vec::new(),
        );
        return;
    }

    let status = strict_status(
        config::strict_path_conflicts().unwrap_or(StrictMode::Warn),
        warnings,
        issues,
    );
    let mut details = conflicts;
    details.push("Keep ~/.vex/bin ahead of legacy manager bins in PATH.".to_string());
    push_check(
        checks,
        "path_conflicts",
        status,
        "PATH contains legacy manager bins ahead of ~/.vex/bin",
        details,
    );
}

fn collect_captured_env_check(
    vex_dir: &Path,
    warnings: &mut usize,
    issues: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) {
    let expected_prefix = vex_dir.to_string_lossy().to_string();
    let mut mismatches = Vec::new();
    for key in [
        "CARGO_HOME",
        "GOPATH",
        "GOBIN",
        "GOMODCACHE",
        "GOCACHE",
        "NPM_CONFIG_CACHE",
        "NPM_CONFIG_PREFIX",
        "COREPACK_HOME",
        "PNPM_HOME",
        "YARN_CACHE_FOLDER",
        "PIP_CACHE_DIR",
    ] {
        if let Ok(value) = std::env::var(key) {
            if !value.starts_with(&expected_prefix) {
                mismatches.push(format!("{}={}", key, value));
            }
        }
    }

    if mismatches.is_empty() {
        push_check(
            checks,
            "captured_env",
            CheckStatus::Ok,
            "captured language home/cache variables point into ~/.vex",
            Vec::new(),
        );
        return;
    }

    let status = strict_status(
        config::strict_path_conflicts().unwrap_or(StrictMode::Warn),
        warnings,
        issues,
    );
    let mut details = mismatches;
    details.push("Re-open your shell with 'eval \"$(vex env <shell>)\"' and run 'vex repair migrate-home' if needed.".to_string());
    push_check(
        checks,
        "captured_env",
        status,
        "some captured language home/cache variables still point outside ~/.vex",
        details,
    );
}

fn collect_manager_conflict_check(
    warnings: &mut usize,
    issues: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) {
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return,
    };
    let managers = [
        (".asdf", "asdf"),
        (".mise", "mise"),
        (".nvm", "nvm"),
        (".rustup", "rustup"),
        (".pyenv", "pyenv"),
    ];
    let present = managers
        .into_iter()
        .filter_map(|(path, label)| home.join(path).exists().then_some(label.to_string()))
        .collect::<Vec<_>>();

    if present.is_empty() {
        push_check(
            checks,
            "manager_conflicts",
            CheckStatus::Ok,
            "no common conflicting version manager homes were detected",
            Vec::new(),
        );
        return;
    }

    let status = strict_status(
        config::strict_path_conflicts().unwrap_or(StrictMode::Warn),
        warnings,
        issues,
    );
    let mut details = present;
    details.push(
        "These tools can coexist with vex, but they may still own files outside ~/.vex."
            .to_string(),
    );
    push_check(
        checks,
        "manager_conflicts",
        status,
        "other version-manager homes were detected",
        details,
    );
}

fn collect_python_base_check(vex_dir: &Path, warnings: &mut usize, checks: &mut Vec<DoctorCheck>) {
    let current_versions = match version_state::read_current_versions(vex_dir) {
        Ok(versions) => versions,
        Err(_) => return,
    };
    let Some(version) = current_versions.get("python") else {
        push_check(
            checks,
            "python_base_env",
            CheckStatus::Ok,
            "python base environment check skipped because Python is not active",
            Vec::new(),
        );
        return;
    };

    let base_dir = python::base_env_dir(vex_dir, version);
    let base_bin = python::base_bin_dir(vex_dir, version);
    let mut details = vec![format!("Base: {}", base_dir.display())];
    let mut status = CheckStatus::Ok;
    let mut summary = "python base environment is ready".to_string();

    if !python::is_base_env_healthy(vex_dir, version) {
        *warnings += 1;
        status = CheckStatus::Warn;
        summary = "python base environment is missing or incomplete".to_string();
        details.push(format!(
            "Run 'vex python base' to create the base environment for python@{}.",
            version
        ));
    }

    if std::env::var_os("VIRTUAL_ENV").is_some() {
        let base_bin_str = base_bin.to_string_lossy().to_string();
        let leaks_into_venv = std::env::var("PATH")
            .unwrap_or_default()
            .split(':')
            .any(|entry| entry == base_bin_str);
        if leaks_into_venv {
            if status != CheckStatus::Warn {
                *warnings += 1;
            }
            status = CheckStatus::Warn;
            summary = "python base bin is active inside a virtual environment".to_string();
            details.push(
                "Project virtual environments should not inherit Python base CLI packages."
                    .to_string(),
            );
        }
    }

    push_check(checks, "python_base_env", status, &summary, details);
}

fn strict_status(mode: StrictMode, warnings: &mut usize, issues: &mut usize) -> CheckStatus {
    match mode {
        StrictMode::Warn => {
            *warnings += 1;
            CheckStatus::Warn
        }
        StrictMode::Enforce => {
            *issues += 1;
            CheckStatus::Error
        }
    }
}
