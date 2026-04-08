mod hooks;
mod path_priority;

use crate::commands::doctor::types::DoctorCheck;
use std::path::Path;

pub(super) fn collect_shell_hook_check(shell: &str) -> DoctorCheck {
    hooks::collect_shell_hook_check(shell)
}

pub(super) fn collect_duplicate_hook_check(shell: &str) -> DoctorCheck {
    hooks::collect_duplicate_hook_check(shell)
}

pub(super) fn collect_path_priority_check(vex_bin: &Path) -> DoctorCheck {
    path_priority::collect_path_priority_check(vex_bin)
}

pub(super) fn collect_tool_manager_conflict_check(vex_bin: &Path) -> DoctorCheck {
    path_priority::collect_tool_manager_conflict_check(vex_bin)
}
