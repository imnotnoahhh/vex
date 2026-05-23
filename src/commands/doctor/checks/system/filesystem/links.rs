use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn collect_broken_links(vex_dir: &Path) -> (Vec<String>, bool) {
    let mut broken_links = Vec::new();
    let mut corepack_missing = false;

    collect_broken_link_entries(
        &vex_dir.join("current"),
        "current",
        &mut broken_links,
        &mut corepack_missing,
    );
    collect_broken_link_entries(
        &vex_dir.join("bin"),
        "bin",
        &mut broken_links,
        &mut corepack_missing,
    );

    (broken_links, corepack_missing)
}

/// Walk `~/.vex/current` and `~/.vex/bin`, delete any symlink whose target
/// no longer exists, and return the labels (e.g. `bin/node`) of what was removed.
pub(crate) fn repair_broken_links(vex_dir: &Path) -> Vec<String> {
    let mut removed = Vec::new();
    repair_broken_link_entries(&vex_dir.join("current"), "current", &mut removed);
    repair_broken_link_entries(&vex_dir.join("bin"), "bin", &mut removed);
    removed
}

fn repair_broken_link_entries(dir: &Path, prefix: &str, removed: &mut Vec<String>) {
    for path in iter_broken_links(dir) {
        let label = path
            .file_name()
            .map(|n| format!("{}/{}", prefix, n.to_string_lossy()))
            .unwrap_or_else(|| format!("{}/<unknown>", prefix));
        if fs::remove_file(&path).is_ok() {
            removed.push(label);
        }
    }
}

fn iter_broken_links(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !dir.exists() {
        return out;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();
        // Must be a symlink AND must fail to canonicalize (target gone).
        if fs::read_link(&path).is_ok() && path.canonicalize().is_err() {
            out.push(path);
        }
    }
    out
}

fn collect_broken_link_entries(
    dir: &Path,
    prefix: &str,
    broken_links: &mut Vec<String>,
    corepack_missing: &mut bool,
) {
    if !dir.exists() {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.filter_map(|entry| entry.ok()) {
        if fs::read_link(entry.path()).is_err() || entry.path().canonicalize().is_ok() {
            continue;
        }

        let filename = entry.file_name().to_string_lossy().to_string();
        if prefix == "bin" && filename == "corepack" {
            *corepack_missing = true;
        } else {
            broken_links.push(format!("{}/{}", prefix, filename));
        }
    }
}
