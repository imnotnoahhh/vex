mod analysis;
mod environment;
mod runtime;
mod summary;
mod system;

use super::types::DoctorReport;
use crate::config;
use crate::error::{Result, VexError};
use crate::resolver;
use crate::version_state;

pub(super) fn collect() -> Result<DoctorReport> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let mut issues = 0;
    let mut warnings = 0;
    let mut checks = Vec::new();

    let vex_bin = config::bin_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    environment::collect_environment_checks(
        &vex_dir,
        &vex_bin,
        &mut warnings,
        &mut issues,
        &mut checks,
    );
    runtime::collect_runtime_checks(&vex_dir, &vex_bin, &mut warnings, &mut issues, &mut checks)?;

    let cwd = resolver::current_dir();
    let retained = version_state::retained_versions(&vex_dir, &cwd)?;
    let disk_usage = analysis::collect_disk_usage(&vex_dir)?;
    let unused_versions = analysis::collect_unused_versions(&vex_dir, &retained)?;
    let lifecycle_warnings = analysis::collect_lifecycle_warnings(&vex_dir)?;

    let total_disk_bytes = disk_usage.iter().map(|u| u.total_bytes).sum();
    let reclaimable_bytes = unused_versions.iter().map(|u| u.bytes).sum();
    let suggestions = summary::build_suggestions(
        unused_versions.len(),
        &lifecycle_warnings,
        issues,
        checks.iter().any(|check| {
            check.id == "home_hygiene" && check.status != super::types::CheckStatus::Ok
        }),
        checks.iter().any(|check| {
            (check.id == "path_conflicts" || check.id == "captured_env")
                && check.status != super::types::CheckStatus::Ok
        }),
    );

    Ok(DoctorReport {
        root: vex_dir.display().to_string(),
        issues,
        warnings,
        checks,
        disk_usage,
        unused_versions,
        lifecycle_warnings,
        total_disk_bytes,
        reclaimable_bytes,
        suggestions,
    })
}
