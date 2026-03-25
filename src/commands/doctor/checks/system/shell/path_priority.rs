use crate::commands::doctor::types::{CheckStatus, DoctorCheck};
use std::path::Path;

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
