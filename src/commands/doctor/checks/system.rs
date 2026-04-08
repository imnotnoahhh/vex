mod config;
mod filesystem;
mod shell;

use super::super::types::DoctorCheck;
use std::path::Path;

pub(super) fn collect_shell_hook_check(shell_name: &str) -> DoctorCheck {
    shell::collect_shell_hook_check(shell_name)
}

pub(super) fn collect_duplicate_hook_check(shell_name: &str) -> DoctorCheck {
    shell::collect_duplicate_hook_check(shell_name)
}

pub(super) fn collect_path_priority_check(vex_bin: &Path) -> DoctorCheck {
    shell::collect_path_priority_check(vex_bin)
}

pub(super) fn collect_tool_manager_conflict_check(vex_bin: &Path) -> DoctorCheck {
    shell::collect_tool_manager_conflict_check(vex_bin)
}

pub(super) fn collect_config_check(vex_dir: &Path) -> DoctorCheck {
    config::collect_config_check(vex_dir)
}

pub(super) fn collect_tool_versions_file_check(path: &Path) -> DoctorCheck {
    config::collect_tool_versions_file_check(path)
}

pub(super) fn collect_project_config_check() -> DoctorCheck {
    config::collect_project_config_check()
}

pub(super) fn collect_effective_settings_check() -> DoctorCheck {
    config::collect_effective_settings_check()
}

pub(super) fn collect_broken_links(vex_dir: &Path) -> (Vec<String>, bool) {
    filesystem::collect_broken_links(vex_dir)
}

pub(super) fn collect_failed_binaries(bin_dir: &Path) -> Vec<String> {
    filesystem::collect_failed_binaries(bin_dir)
}

pub(super) fn collect_cache_integrity_check(vex_dir: &Path) -> DoctorCheck {
    filesystem::collect_cache_integrity_check(vex_dir)
}
