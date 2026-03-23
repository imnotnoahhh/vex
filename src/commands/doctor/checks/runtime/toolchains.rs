use super::super::super::types::{push_check, CheckStatus, DoctorCheck};
use super::super::system;
use std::fs;
use std::path::Path;

pub(super) fn push_installed_tools_check(
    checks: &mut Vec<DoctorCheck>,
    toolchains_dir: &Path,
    warnings: &mut usize,
    issues: &mut usize,
) {
    let tool_count = count_installed_toolchains(toolchains_dir);
    push_check(
        checks,
        "installed_tools",
        if toolchains_dir.exists() && tool_count > 0 {
            CheckStatus::Ok
        } else if toolchains_dir.exists() {
            *warnings += 1;
            CheckStatus::Warn
        } else {
            *issues += 1;
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
}

pub(super) fn push_symlink_check(
    checks: &mut Vec<DoctorCheck>,
    vex_dir: &Path,
    warnings: &mut usize,
) {
    let (broken_links, corepack_missing) = system::collect_broken_links(vex_dir);
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
        *warnings += 1;
        DoctorCheck {
            id: "symlinks".to_string(),
            status: CheckStatus::Warn,
            summary: "broken symlinks were found".to_string(),
            details: broken_links,
        }
    };
    checks.push(symlink_check);
}

fn count_installed_toolchains(toolchains_dir: &Path) -> usize {
    if !toolchains_dir.exists() {
        return 0;
    }

    fs::read_dir(toolchains_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry
                        .file_type()
                        .ok()
                        .map(|file_type| file_type.is_dir())
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}
