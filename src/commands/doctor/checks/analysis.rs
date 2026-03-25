mod lifecycle;
mod usage;

use super::super::types::{LifecycleWarning, ToolDiskUsage, UnusedVersion};
use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;

pub(super) fn collect_disk_usage(vex_dir: &Path) -> Result<Vec<ToolDiskUsage>> {
    usage::collect_disk_usage(vex_dir)
}

pub(super) fn collect_unused_versions(
    vex_dir: &Path,
    retained: &HashMap<(String, String), String>,
) -> Result<Vec<UnusedVersion>> {
    usage::collect_unused_versions(vex_dir, retained)
}

pub(super) fn collect_lifecycle_warnings(vex_dir: &Path) -> Result<Vec<LifecycleWarning>> {
    lifecycle::collect_lifecycle_warnings(vex_dir)
}
