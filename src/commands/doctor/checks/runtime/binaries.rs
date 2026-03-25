use super::super::super::types::{push_check, CheckStatus, DoctorCheck};
use super::super::system;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(super) fn push_binary_permissions_check(
    checks: &mut Vec<DoctorCheck>,
    vex_bin: &Path,
    warnings: &mut usize,
) {
    let non_executable = fs::read_dir(vex_bin)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            if metadata.is_symlink() || (metadata.permissions().mode() & 0o111) != 0 {
                return None;
            }
            Some(entry.file_name().to_string_lossy().to_string())
        })
        .collect::<Vec<_>>();

    push_check(
        checks,
        "binary_permissions",
        if non_executable.is_empty() {
            CheckStatus::Ok
        } else {
            *warnings += 1;
            CheckStatus::Warn
        },
        if non_executable.is_empty() {
            "vex-managed binaries are executable"
        } else {
            "some vex-managed binaries are not executable"
        },
        non_executable,
    );
}

pub(super) fn push_binary_runnability_check(
    checks: &mut Vec<DoctorCheck>,
    vex_bin: &Path,
    warnings: &mut usize,
) {
    let failed_binaries = system::collect_failed_binaries(vex_bin);
    push_check(
        checks,
        "binary_runnability",
        if failed_binaries.is_empty() {
            CheckStatus::Ok
        } else {
            *warnings += 1;
            CheckStatus::Warn
        },
        if failed_binaries.is_empty() {
            "managed binaries respond to probe commands"
        } else {
            "some binaries did not respond to probe commands"
        },
        failed_binaries,
    );
}
