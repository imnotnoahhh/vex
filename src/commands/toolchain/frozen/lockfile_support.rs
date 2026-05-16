use crate::error::{Result, VexError};
use crate::lockfile;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub(super) struct LockedTool {
    pub tool: String,
    pub version: String,
    pub sha256: String,
}

pub(super) fn load_lockfile_for_frozen(cwd: &Path) -> Result<lockfile::Lockfile> {
    lockfile::Lockfile::load_from_ancestors(cwd)?.ok_or_else(|| {
        VexError::Config(
            "Frozen mode requires a lockfile (.tool-versions.lock). Run 'vex lock' first."
                .to_string(),
        )
    })
}

pub(super) fn validate_lockfile_matches_versions(
    lockfile: &lockfile::Lockfile,
    versions: &HashMap<String, String>,
) -> Result<()> {
    if versions.is_empty() {
        return Err(VexError::Config(
            super::NO_VERSION_FILES_MESSAGE.to_string(),
        ));
    }

    for (tool_name, version) in versions {
        if let Some(lock_entry) = lockfile.get_tool(tool_name) {
            if &lock_entry.version != version {
                return Err(VexError::Config(format!(
                    "Version mismatch for {}: .tool-versions specifies '{}' but lockfile has '{}'. Update lockfile with 'vex lock' or remove --frozen flag.",
                    tool_name, version, lock_entry.version
                )));
            }
        } else {
            return Err(VexError::Config(format!(
                "Tool '{}' found in .tool-versions but not in lockfile. Update lockfile with 'vex lock' or remove --frozen flag.",
                tool_name
            )));
        }
    }

    Ok(())
}

pub(super) fn locked_tools(lockfile: &lockfile::Lockfile) -> Result<Vec<LockedTool>> {
    lockfile
        .tools
        .iter()
        .map(|(tool, entry)| {
            let sha256 = entry.sha256.as_ref().ok_or_else(|| {
                VexError::Config(format!(
                    "Lockfile entry for {}@{} is missing sha256. Run 'vex lock' with a verified install before using --frozen.",
                    tool, entry.version
                ))
            })?;
            if sha256.len() != 64 || !sha256.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return Err(VexError::Config(format!(
                    "Lockfile entry for {}@{} has an invalid sha256 value.",
                    tool, entry.version
                )));
            }

            Ok(LockedTool {
                tool: tool.clone(),
                version: entry.version.clone(),
                sha256: sha256.to_ascii_lowercase(),
            })
        })
        .collect()
}

pub(super) fn verify_installed_checksum(
    vex_dir: &Path,
    tool_name: &str,
    version: &str,
    expected: &str,
) -> Result<()> {
    let checksum_file = vex_dir
        .join("toolchains")
        .join(tool_name)
        .join(version)
        .join(".vex-checksum");
    let actual = fs::read_to_string(&checksum_file).map_err(|_| {
        VexError::Config(format!(
            "{}@{} is installed but has no recorded checksum. Reinstall it with --frozen so vex can verify lockfile integrity.",
            tool_name, version
        ))
    })?;
    let actual = actual.trim().to_ascii_lowercase();
    if actual == expected {
        Ok(())
    } else {
        Err(VexError::Config(format!(
            "{}@{} checksum does not match lockfile (expected {}, found {}). Reinstall the toolchain from a verified archive.",
            tool_name, version, expected, actual
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lockfile::{LockEntry, Lockfile};

    #[test]
    fn test_locked_tools_requires_sha256() {
        let mut lockfile = Lockfile::new();
        lockfile.add_tool(
            "node".to_string(),
            LockEntry {
                version: "20.11.0".to_string(),
                sha256: None,
                url: None,
            },
        );

        let error = locked_tools(&lockfile).unwrap_err();
        assert!(error.to_string().contains("missing sha256"));
    }

    #[test]
    fn test_locked_tools_normalizes_sha256() {
        let mut lockfile = Lockfile::new();
        lockfile.add_tool(
            "node".to_string(),
            LockEntry {
                version: "20.11.0".to_string(),
                sha256: Some("A".repeat(64)),
                url: None,
            },
        );

        let locked = locked_tools(&lockfile).unwrap();
        assert_eq!(locked[0].sha256, "a".repeat(64));
    }
}
