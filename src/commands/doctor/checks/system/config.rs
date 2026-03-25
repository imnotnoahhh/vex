mod effective;
mod files;
mod project;

use crate::commands::doctor::types::DoctorCheck;
use std::path::Path;

pub(super) fn collect_config_check(vex_dir: &Path) -> DoctorCheck {
    files::collect_config_check(vex_dir)
}

pub(super) fn collect_tool_versions_file_check(path: &Path) -> DoctorCheck {
    files::collect_tool_versions_file_check(path)
}

pub(super) fn collect_project_config_check() -> DoctorCheck {
    project::collect_project_config_check()
}

pub(super) fn collect_effective_settings_check() -> DoctorCheck {
    effective::collect_effective_settings_check()
}
