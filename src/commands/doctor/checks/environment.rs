use super::super::types::{push_check, CheckStatus, DoctorCheck};
use super::system;
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
}
