mod binaries;
mod cache;
mod links;

use crate::commands::doctor::types::DoctorCheck;
use std::path::Path;

pub(super) fn collect_broken_links(vex_dir: &Path) -> (Vec<String>, bool) {
    links::collect_broken_links(vex_dir)
}

pub(super) fn collect_failed_binaries(bin_dir: &Path) -> Vec<String> {
    binaries::collect_failed_binaries(bin_dir)
}

pub(super) fn collect_cache_integrity_check(vex_dir: &Path) -> DoctorCheck {
    cache::collect_cache_integrity_check(vex_dir)
}
