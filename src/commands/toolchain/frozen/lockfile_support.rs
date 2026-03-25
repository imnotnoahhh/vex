use crate::error::{Result, VexError};
use crate::lockfile;
use std::collections::HashMap;
use std::path::Path;

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
