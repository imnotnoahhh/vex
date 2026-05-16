use super::{checked_install_dir, push_path_entry};
use crate::error::Result;
use crate::project;
use crate::tools;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(in crate::activation) fn collect_shared_path_entries(
    cwd: &Path,
    vex_dir: &Path,
    toolchains_dir: &Path,
    versions: &BTreeMap<String, String>,
    venv_dir: Option<&Path>,
    capture_user_state: bool,
) -> Result<Vec<PathBuf>> {
    let mut path_entries = Vec::new();
    let mut path_seen = BTreeSet::new();

    if let Some(venv_dir) = venv_dir {
        push_path_entry(&mut path_entries, &mut path_seen, venv_dir.join("bin"));
    }

    if versions.contains_key("node") {
        if let Some(node_modules_bin) = project::find_nearest_node_modules_bin(cwd) {
            push_path_entry(&mut path_entries, &mut path_seen, node_modules_bin);
        }
    }

    if capture_user_state {
        for (tool_name, version) in versions {
            if tool_name == "python" && venv_dir.is_some() {
                continue;
            }

            let tool = tools::get_tool(tool_name)?;
            let install_dir = checked_install_dir(toolchains_dir, tool_name, version)?;
            let environment = tool.managed_environment(vex_dir, Some(&install_dir));
            for path in environment.managed_user_bin_dirs {
                push_path_entry(&mut path_entries, &mut path_seen, PathBuf::from(path));
            }
        }
    }

    Ok(path_entries)
}

pub(in crate::activation) fn collect_exec_path_entries(
    toolchains_dir: &Path,
    versions: &BTreeMap<String, String>,
) -> Result<Vec<PathBuf>> {
    let mut path_entries = Vec::new();
    let mut path_seen = BTreeSet::new();

    for (tool_name, version) in versions {
        append_tool_bin_paths(
            &mut path_entries,
            &mut path_seen,
            toolchains_dir,
            tool_name,
            version,
        )?;
    }

    Ok(path_entries)
}

pub(in crate::activation) fn merged_path(path_entries: &[PathBuf], original_path: &str) -> String {
    let mut merged = Vec::new();
    let mut seen = BTreeSet::new();

    for entry in path_entries {
        let segment = entry.display().to_string();
        if !segment.is_empty() && seen.insert(segment.clone()) {
            merged.push(segment);
        }
    }

    for segment in original_path
        .split(':')
        .filter(|segment| !segment.is_empty())
    {
        let segment = segment.to_string();
        if seen.insert(segment.clone()) {
            merged.push(segment);
        }
    }

    merged.join(":")
}

pub(in crate::activation) fn original_path() -> String {
    std::env::var("VEX_ORIGINAL_PATH")
        .or_else(|_| std::env::var("PATH"))
        .unwrap_or_default()
}

fn append_tool_bin_paths(
    path_entries: &mut Vec<PathBuf>,
    path_seen: &mut BTreeSet<PathBuf>,
    toolchains_dir: &Path,
    tool_name: &str,
    version: &str,
) -> Result<()> {
    let tool = tools::get_tool(tool_name)?;
    let install_dir = checked_install_dir(toolchains_dir, tool_name, version)?;

    let mut local_seen = BTreeSet::new();
    for (_, subpath) in tool.bin_paths() {
        let bin_dir = install_dir.join(subpath);
        if bin_dir.exists() && local_seen.insert(bin_dir.clone()) {
            push_path_entry(path_entries, path_seen, bin_dir);
        }
    }

    Ok(())
}
