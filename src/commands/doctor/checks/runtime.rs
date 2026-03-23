mod binaries;
mod network;
mod toolchains;

use super::super::types::{CheckStatus, DoctorCheck};
use super::system;
use crate::config;
use crate::error::{Result, VexError};
use binaries::{push_binary_permissions_check, push_binary_runnability_check};
use network::push_network_check;
use std::path::Path;
use toolchains::{push_installed_tools_check, push_symlink_check};

pub(super) fn collect_runtime_checks(
    vex_dir: &Path,
    vex_bin: &Path,
    warnings: &mut usize,
    issues: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) -> Result<()> {
    let toolchains_dir = config::toolchains_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    push_installed_tools_check(checks, &toolchains_dir, warnings, issues);
    push_symlink_check(checks, vex_dir, warnings);
    push_binary_permissions_check(checks, vex_bin, warnings);
    push_binary_runnability_check(checks, vex_bin, warnings);

    let cache_check = system::collect_cache_integrity_check(vex_dir);
    if cache_check.status == CheckStatus::Warn {
        *warnings += 1;
    }
    checks.push(cache_check);

    push_network_check(checks, warnings);
    Ok(())
}
