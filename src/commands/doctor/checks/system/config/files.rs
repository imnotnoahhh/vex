use crate::commands::doctor::types::{CheckStatus, DoctorCheck};
use crate::config;
use std::fs;
use std::path::Path;

pub(super) fn collect_config_check(vex_dir: &Path) -> DoctorCheck {
    let config_path = vex_dir.join("config.toml");
    if !config_path.exists() {
        return DoctorCheck {
            id: "config".to_string(),
            status: CheckStatus::Warn,
            summary: "config.toml is missing".to_string(),
            details: vec!["Run 'vex init' to recreate ~/.vex/config.toml".to_string()],
        };
    }

    match config::load_settings_from_file(&config_path) {
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

pub(super) fn collect_tool_versions_file_check(path: &Path) -> DoctorCheck {
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

    let invalid_lines = content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| invalid_tool_versions_line(index, line))
        .collect::<Vec<_>>();

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

fn invalid_tool_versions_line(index: usize, line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    let parts = line.split_whitespace().collect::<Vec<_>>();
    if parts.len() == 2 {
        None
    } else {
        Some(format!("Line {}: {}", index + 1, line))
    }
}
