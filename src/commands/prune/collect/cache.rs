use crate::commands::prune::RemovalCandidate;
use crate::error::Result;
use crate::fs_utils::path_size;
use std::fs;
use std::path::Path;

pub(super) fn cache_candidates(vex_dir: &Path) -> Result<Vec<RemovalCandidate>> {
    let cache_dir = vex_dir.join("cache");
    if !cache_dir.exists() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&cache_dir)?.filter_map(|e| e.ok()) {
        let path = entry.path();
        candidates.push(RemovalCandidate {
            kind: "cache".to_string(),
            bytes: path_size(&path),
            path: path.display().to_string(),
        });
    }

    Ok(candidates)
}
