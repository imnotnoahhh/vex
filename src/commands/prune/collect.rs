mod cache;
mod locks;
mod toolchains;

use super::{PruneReport, RetainedToolchain};
use crate::config;
use crate::error::{Result, VexError};
use crate::resolver;
use crate::version_state;
use std::path::Path;
use std::time::SystemTime;

pub(super) fn collect_plan(dry_run: bool) -> Result<PruneReport> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let cwd = resolver::current_dir();
    collect_plan_for(&vex_dir, &cwd, dry_run, SystemTime::now())
}

fn collect_plan_for(
    vex_dir: &Path,
    cwd: &Path,
    dry_run: bool,
    now: SystemTime,
) -> Result<PruneReport> {
    let retained = version_state::retained_versions(vex_dir, cwd)?;
    let mut retained_toolchains = retained
        .iter()
        .map(|((tool, version), reason)| RetainedToolchain {
            tool: tool.clone(),
            version: version.clone(),
            reason: reason.clone(),
        })
        .collect::<Vec<_>>();
    retained_toolchains.sort_by(|a, b| a.tool.cmp(&b.tool).then(a.version.cmp(&b.version)));

    let mut removable = Vec::new();
    removable.extend(cache::cache_candidates(vex_dir)?);
    removable.extend(locks::stale_lock_candidates(vex_dir, now)?);
    removable.extend(toolchains::unused_toolchain_candidates(vex_dir, &retained)?);

    let total_bytes = removable.iter().map(|item| item.bytes).sum();
    let total_candidates = removable.len();

    Ok(PruneReport {
        dry_run,
        removable,
        retained_toolchains,
        total_candidates,
        total_bytes,
        note: "Only ~/.vex state is pruned. Current activations, global defaults, and versions pinned in the current working tree are retained. Other repositories are not scanned.".to_string(),
    })
}
