use crate::commands::doctor::types::{CheckStatus, DoctorCheck};
use std::fs;
use std::path::Path;

pub(super) fn collect_cache_integrity_check(vex_dir: &Path) -> DoctorCheck {
    let cache_dir = vex_dir.join("cache");
    if !cache_dir.exists() {
        return DoctorCheck {
            id: "cache_integrity".to_string(),
            status: CheckStatus::Ok,
            summary: "cache directory is absent".to_string(),
            details: Vec::new(),
        };
    }

    let invalid_files = collect_invalid_cache_files(&cache_dir);
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

fn collect_invalid_cache_files(cache_dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return Vec::new();
    };

    entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| invalid_cache_file_message(&entry.path()))
        .collect()
}

fn invalid_cache_file_message(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    if !(name.starts_with("remote-") && name.ends_with(".json")) {
        return None;
    }

    match fs::read_to_string(path) {
        Ok(content) if serde_json::from_str::<serde_json::Value>(&content).is_ok() => None,
        Ok(_) => Some(path.display().to_string()),
        Err(err) => Some(format!("{} ({})", path.display(), err)),
    }
}
