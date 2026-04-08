use crate::commands::doctor::types::{CheckStatus, DoctorCheck};
use std::path::Path;

fn managed_npm_bin(vex_bin: &Path) -> Option<String> {
    vex_bin.parent().map(|parent| {
        parent
            .join("npm")
            .join("prefix")
            .join("bin")
            .to_string_lossy()
            .to_string()
    })
}

fn detect_tool_manager(entry: &str) -> Option<&'static str> {
    if entry.contains("/.pyenv/shims") {
        Some("pyenv shims")
    } else if entry.contains("/.pyenv/bin") {
        Some("pyenv")
    } else if entry.contains("/.nvm/") {
        Some("nvm")
    } else if entry.contains("/.fnm/") {
        Some("fnm")
    } else if entry.contains("/.volta/bin") {
        Some("volta")
    } else if entry.contains("/.asdf/") {
        Some("asdf")
    } else if entry.contains("/.cargo/bin") && !entry.contains("/.vex/cargo/bin") {
        Some("rustup cargo env")
    } else if entry.contains("/.vex/cargo/bin") {
        Some("cargo env")
    } else {
        None
    }
}

pub(super) fn collect_path_priority_check(vex_bin: &Path) -> DoctorCheck {
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

    let managed_npm_bin = managed_npm_bin(vex_bin);
    let unexpected_before = path_entries[..index]
        .iter()
        .filter(|entry| {
            managed_npm_bin
                .as_deref()
                .map(|npm_bin| **entry != npm_bin)
                .unwrap_or(true)
        })
        .copied()
        .collect::<Vec<_>>();

    if unexpected_before.is_empty() {
        DoctorCheck {
            id: "path_priority".to_string(),
            status: CheckStatus::Ok,
            summary: if index == 0 {
                "vex/bin has first PATH priority".to_string()
            } else {
                "managed vex paths lead PATH".to_string()
            },
            details: if index == 0 {
                Vec::new()
            } else {
                vec!["~/.vex/npm/prefix/bin appears before ~/.vex/bin, which is expected for npm global binaries".to_string()]
            },
        }
    } else {
        DoctorCheck {
            id: "path_priority".to_string(),
            status: CheckStatus::Warn,
            summary: "vex/bin is present in PATH but shadowed by earlier entries".to_string(),
            details: vec![
                format!(
                    "~/.vex/bin appears at PATH position {}. Earlier entries may shadow managed binaries.",
                    index + 1
                ),
                format!("Earlier entries: {}", unexpected_before.join(": ")),
            ],
        }
    }
}

pub(super) fn collect_tool_manager_conflict_check(vex_bin: &Path) -> DoctorCheck {
    let Ok(path_var) = std::env::var("PATH") else {
        return DoctorCheck {
            id: "tool_manager_conflicts".to_string(),
            status: CheckStatus::Ok,
            summary: "tool manager conflict check skipped because PATH is unavailable".to_string(),
            details: Vec::new(),
        };
    };

    let path_entries = path_var.split(':').collect::<Vec<_>>();
    let vex_bin_str = vex_bin.to_string_lossy().to_string();
    let Some(index) = path_entries.iter().position(|entry| *entry == vex_bin_str) else {
        return DoctorCheck {
            id: "tool_manager_conflicts".to_string(),
            status: CheckStatus::Ok,
            summary: "tool manager conflict check skipped until vex/bin is on PATH".to_string(),
            details: Vec::new(),
        };
    };

    let conflicts = path_entries[..index]
        .iter()
        .filter_map(|entry| detect_tool_manager(entry).map(|manager| (*entry, manager)))
        .collect::<Vec<_>>();

    if conflicts.is_empty() {
        DoctorCheck {
            id: "tool_manager_conflicts".to_string(),
            status: CheckStatus::Ok,
            summary: "no active tool manager PATH conflicts detected".to_string(),
            details: Vec::new(),
        }
    } else {
        let mut details = conflicts
            .iter()
            .map(|(entry, manager)| format!("{} is active before ~/.vex/bin ({})", entry, manager))
            .collect::<Vec<_>>();
        details.push(
            "These paths can shadow vex-managed binaries. Move ~/.vex/npm/prefix/bin and ~/.vex/bin earlier in PATH, or disable the competing shell init. vex does not migrate these managers automatically.".to_string(),
        );

        DoctorCheck {
            id: "tool_manager_conflicts".to_string(),
            status: CheckStatus::Warn,
            summary: "tool manager paths are active before vex/bin".to_string(),
            details,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_path<T>(path: Option<&str>, f: impl FnOnce() -> T) -> T {
        let _guard = ENV_LOCK.lock().unwrap();
        let original = std::env::var("PATH").ok();

        if let Some(path) = path {
            std::env::set_var("PATH", path);
        } else {
            std::env::remove_var("PATH");
        }

        let result = f();

        if let Some(original) = original {
            std::env::set_var("PATH", original);
        } else {
            std::env::remove_var("PATH");
        }

        result
    }

    #[test]
    fn test_path_priority_accepts_managed_npm_bin_before_vex_bin() {
        let vex_bin = Path::new("/Users/test/.vex/bin");
        let check = with_path(
            Some("/Users/test/.vex/npm/prefix/bin:/Users/test/.vex/bin:/usr/bin"),
            || collect_path_priority_check(vex_bin),
        );
        assert_eq!(check.status, CheckStatus::Ok);
    }

    #[test]
    fn test_path_priority_warns_on_shadowing_entries() {
        let vex_bin = Path::new("/Users/test/.vex/bin");
        let check = with_path(
            Some("/Users/test/.vex/cargo/bin:/Users/test/.vex/bin:/usr/bin"),
            || collect_path_priority_check(vex_bin),
        );
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.summary.contains("shadowed"));
    }

    #[test]
    fn test_tool_manager_conflict_detects_pyenv_before_vex_bin() {
        let vex_bin = Path::new("/Users/test/.vex/bin");
        let check = with_path(
            Some("/Users/test/.pyenv/shims:/Users/test/.vex/bin:/usr/bin"),
            || collect_tool_manager_conflict_check(vex_bin),
        );
        assert_eq!(check.status, CheckStatus::Warn);
        assert!(check.details.iter().any(|detail| detail.contains("pyenv")));
    }

    #[test]
    fn test_tool_manager_conflict_skips_when_vex_bin_missing() {
        let vex_bin = Path::new("/Users/test/.vex/bin");
        let check = with_path(Some("/usr/bin:/bin"), || {
            collect_tool_manager_conflict_check(vex_bin)
        });
        assert_eq!(check.status, CheckStatus::Ok);
        assert!(check.summary.contains("skipped"));
    }
}
