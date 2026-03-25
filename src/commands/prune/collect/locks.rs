use crate::commands::prune::{RemovalCandidate, STALE_LOCK_AGE};
use crate::error::Result;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

pub(super) fn stale_lock_candidates(
    vex_dir: &Path,
    now: SystemTime,
) -> Result<Vec<RemovalCandidate>> {
    let locks_dir = vex_dir.join("locks");
    if !locks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&locks_dir)?.filter_map(|e| e.ok()) {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        let Ok(age) = now.duration_since(modified) else {
            continue;
        };
        if age >= STALE_LOCK_AGE {
            candidates.push(RemovalCandidate {
                kind: "stale_lock".to_string(),
                path: path.display().to_string(),
                bytes: metadata.len(),
            });
        }
    }

    Ok(candidates)
}
