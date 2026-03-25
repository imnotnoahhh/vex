use crate::error::Result;
use crate::tools::{self, Tool};
use crate::versioning::{normalize_version, version_sort_key};
use std::fs;
use std::path::Path;

pub(crate) fn resolve_for_install(tool: &dyn Tool, requested: &str) -> Result<String> {
    tools::resolve_fuzzy_version(tool, requested)
}

pub(crate) fn resolve_installed_version(
    vex_dir: &Path,
    tool_name: &str,
    requested: &str,
) -> Result<Option<String>> {
    let tool_dir = vex_dir.join("toolchains").join(tool_name);
    if !tool_dir.exists() {
        return Ok(None);
    }

    let mut matches = fs::read_dir(&tool_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_type()
                .ok()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
        })
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|installed| version_matches_request(installed, requested))
        .collect::<Vec<_>>();

    matches.sort_by_key(|version| version_sort_key(version));
    Ok(matches.pop())
}

pub(crate) fn version_matches_request(installed_version: &str, requested: &str) -> bool {
    let installed = normalize_version(installed_version);
    let requested = normalize_version(requested);
    installed == requested || installed.starts_with(&format!("{requested}."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn matches_exact_and_partial_requests() {
        assert!(version_matches_request("20.20.1", "20"));
        assert!(version_matches_request("20.20.1", "20.20"));
        assert!(version_matches_request("20.20.1", "20.20.1"));
        assert!(!version_matches_request("20.20.1", "21"));
        assert!(!version_matches_request("20.20.1", "20.2"));
    }

    #[test]
    fn picks_latest_installed_match_for_partial_requests() {
        let temp = TempDir::new().unwrap();
        let tool_dir = temp.path().join("toolchains").join("node");
        fs::create_dir_all(tool_dir.join("20.9.0")).unwrap();
        fs::create_dir_all(tool_dir.join("20.20.1")).unwrap();
        fs::create_dir_all(tool_dir.join("25.8.0")).unwrap();

        let resolved = resolve_installed_version(temp.path(), "node", "20")
            .unwrap()
            .unwrap();
        assert_eq!(resolved, "20.20.1");
    }
}
