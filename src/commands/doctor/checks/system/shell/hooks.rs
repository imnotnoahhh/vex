use crate::commands::doctor::types::{CheckStatus, DoctorCheck};
use std::fs;
use std::path::PathBuf;

pub(super) fn collect_shell_hook_check(shell: &str) -> DoctorCheck {
    if shell.contains("zsh") {
        return shell_hook_check("zsh", ".zshrc", "vex env zsh", "eval \"$(vex env zsh)\"");
    }
    if shell.contains("bash") {
        return bash_shell_hook_check();
    }

    DoctorCheck {
        id: "shell_hook".to_string(),
        status: CheckStatus::Warn,
        summary: "unable to determine the active shell hook status".to_string(),
        details: vec!["The current shell is not zsh or bash".to_string()],
    }
}

pub(super) fn collect_duplicate_hook_check(shell: &str) -> DoctorCheck {
    let Some((shell_name, file_names, marker)) = shell_hook_target(shell) else {
        return DoctorCheck {
            id: "shell_hook_duplicates".to_string(),
            status: CheckStatus::Warn,
            summary: "unable to check shell hook duplication for this shell".to_string(),
            details: vec!["The current shell is not zsh or bash".to_string()],
        };
    };

    let home = std::env::var("HOME").unwrap_or_default();
    let mut checked_any_file = false;
    let mut count = 0;
    let mut details = Vec::new();
    for file_name in file_names {
        let shell_rc = PathBuf::from(&home).join(file_name);
        let Ok(content) = fs::read_to_string(&shell_rc) else {
            continue;
        };
        checked_any_file = true;
        let file_count = content.matches(marker).count();
        count += file_count;
        if file_count > 0 {
            details.push(format!(
                "Found {} occurrence(s) of '{}' in {}",
                file_count,
                marker,
                shell_rc.display()
            ));
        }
    }

    if !checked_any_file {
        return DoctorCheck {
            id: "shell_hook_duplicates".to_string(),
            status: CheckStatus::Ok,
            summary: format!("{} shell hook duplication could not be checked", shell_name),
            details: Vec::new(),
        };
    }

    if count > 1 {
        DoctorCheck {
            id: "shell_hook_duplicates".to_string(),
            status: CheckStatus::Warn,
            summary: format!("{} shell hook appears multiple times", shell_name),
            details,
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

fn bash_shell_hook_check() -> DoctorCheck {
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [".bashrc", ".bash_profile"];
    let marker = "vex env bash";
    let suggested = "eval \"$(vex env bash)\"";

    let mut unreadable = Vec::new();
    let mut existing = Vec::new();
    for file_name in candidates {
        let path = PathBuf::from(&home).join(file_name);
        if !path.exists() {
            continue;
        }
        existing.push(path.clone());
        match fs::read_to_string(&path) {
            Ok(content) if content.contains(marker) => {
                return DoctorCheck {
                    id: "shell_hook".to_string(),
                    status: CheckStatus::Ok,
                    summary: "bash shell hook is configured".to_string(),
                    details: vec![format!("Found hook in {}", path.display())],
                };
            }
            Ok(_) => {}
            Err(_) => unreadable.push(path),
        }
    }

    if existing.is_empty() {
        let target = PathBuf::from(&home).join(".bash_profile");
        return DoctorCheck {
            id: "shell_hook".to_string(),
            status: CheckStatus::Warn,
            summary: "bash shell config file was not found".to_string(),
            details: vec![format!("Create {} and add {}", target.display(), suggested)],
        };
    }

    if !unreadable.is_empty() && unreadable.len() == existing.len() {
        return DoctorCheck {
            id: "shell_hook".to_string(),
            status: CheckStatus::Warn,
            summary: "bash shell config could not be read".to_string(),
            details: unreadable
                .iter()
                .map(|path| format!("Check permissions for {}", path.display()))
                .collect(),
        };
    }

    let target = existing
        .first()
        .cloned()
        .unwrap_or_else(|| PathBuf::from(&home).join(".bash_profile"));
    DoctorCheck {
        id: "shell_hook".to_string(),
        status: CheckStatus::Warn,
        summary: "bash shell hook is not configured".to_string(),
        details: vec![format!("Add {} to {}", suggested, target.display())],
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

fn shell_hook_target(shell: &str) -> Option<(&'static str, &'static [&'static str], &'static str)> {
    if shell.contains("zsh") {
        return Some(("zsh", &[".zshrc"], "vex env zsh"));
    }
    if shell.contains("bash") {
        return Some(("bash", &[".bashrc", ".bash_profile"], "vex env bash"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn with_home<T>(home: &std::path::Path, f: impl FnOnce() -> T) -> T {
        let _guard = crate::test_env::lock();
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", home);
        let result = f();
        if let Some(value) = old_home {
            std::env::set_var("HOME", value);
        } else {
            std::env::remove_var("HOME");
        }
        result
    }

    #[test]
    fn test_bash_hook_check_accepts_bash_profile() {
        let home = TempDir::new().unwrap();
        fs::write(
            home.path().join(".bash_profile"),
            "eval \"$(vex env bash)\"\n",
        )
        .unwrap();

        with_home(home.path(), || {
            let check = collect_shell_hook_check("/bin/bash");
            assert!(matches!(check.status, CheckStatus::Ok));
            assert!(check
                .details
                .iter()
                .any(|detail| detail.contains(".bash_profile")));
        });
    }

    #[test]
    fn test_bash_duplicate_check_counts_both_bash_files() {
        let home = TempDir::new().unwrap();
        fs::write(home.path().join(".bashrc"), "eval \"$(vex env bash)\"\n").unwrap();
        fs::write(
            home.path().join(".bash_profile"),
            "eval \"$(vex env bash)\"\n",
        )
        .unwrap();

        with_home(home.path(), || {
            let check = collect_duplicate_hook_check("/bin/bash");
            assert!(matches!(check.status, CheckStatus::Warn));
            assert_eq!(check.details.len(), 2);
        });
    }
}
