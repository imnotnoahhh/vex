use crate::commands::doctor::types::{CheckStatus, DoctorCheck};
use std::fs;
use std::path::PathBuf;

pub(super) fn collect_shell_hook_check(shell: &str) -> DoctorCheck {
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

pub(super) fn collect_duplicate_hook_check(shell: &str) -> DoctorCheck {
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

fn shell_hook_target(shell: &str) -> Option<(&'static str, &'static str, &'static str)> {
    if shell.contains("zsh") {
        return Some(("zsh", ".zshrc", "vex env zsh"));
    }
    if shell.contains("bash") {
        return Some(("bash", ".bashrc", "vex env bash"));
    }
    None
}
