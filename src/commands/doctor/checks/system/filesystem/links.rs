use std::fs;
use std::path::Path;

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
