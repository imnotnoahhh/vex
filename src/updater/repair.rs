mod prompt;
mod scan;

use crate::config;
use crate::error::{Result, VexError};
use prompt::{
    confirm_reinstall, repair_versions, report_broken_installations, report_complete,
    report_no_broken_installations, report_non_interactive_skip, report_scan_start, report_skipped,
};
use scan::scan_broken_versions;

/// Detect and repair broken installations from old vex versions (< 1.1.0).
/// Old versions had a symlink bug that created 0-byte npm/npx files.
pub(super) fn detect_and_repair_broken_installations(old_version: &str) -> Result<()> {
    if super::release::version_tuple(old_version) >= (1, 1, 0) {
        return Ok(());
    }

    report_scan_start();

    let toolchains_dir = config::toolchains_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    if !toolchains_dir.exists() {
        return Ok(());
    }

    let broken_versions = scan_broken_versions(&toolchains_dir)?;
    if broken_versions.is_empty() {
        report_no_broken_installations();
        return Ok(());
    }

    report_broken_installations(&broken_versions);

    if config::non_interactive()? {
        report_non_interactive_skip();
        return Ok(());
    }

    if !confirm_reinstall()? {
        report_skipped();
        return Ok(());
    }

    repair_versions(&broken_versions);
    report_complete();
    Ok(())
}
