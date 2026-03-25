mod collect;
mod render;

use crate::error::Result;
use collect::collect_plan;
use render::{render_completed, render_dry_run};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const STALE_LOCK_AGE: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Serialize)]
pub struct RemovalCandidate {
    pub kind: String,
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct RetainedToolchain {
    pub tool: String,
    pub version: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct PruneReport {
    pub dry_run: bool,
    pub removable: Vec<RemovalCandidate>,
    pub retained_toolchains: Vec<RetainedToolchain>,
    pub total_candidates: usize,
    pub total_bytes: u64,
    pub note: String,
}

pub fn run(dry_run: bool) -> Result<()> {
    let report = collect_plan(dry_run)?;

    if dry_run {
        render_dry_run(&report);
        return Ok(());
    }

    for candidate in &report.removable {
        let path = PathBuf::from(&candidate.path);
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else if path.exists() {
            fs::remove_file(&path)?;
        }
    }

    render_completed(&report);
    Ok(())
}
