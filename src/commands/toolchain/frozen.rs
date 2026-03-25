mod install;
mod lockfile_support;
mod sync;

use crate::error::Result;

pub(super) const NO_VERSION_FILES_MESSAGE: &str =
    "No version files found (.tool-versions, .node-version, etc.)";

pub fn install_from_version_files_with_frozen(frozen: bool, offline: bool) -> Result<()> {
    if frozen {
        install::from_lockfile(offline)
    } else {
        install::from_version_files(offline)
    }
}

pub fn sync_from_current_context_with_frozen(frozen: bool, offline: bool) -> Result<()> {
    if frozen {
        sync::from_lockfile(offline)
    } else {
        sync::from_current_context(offline)
    }
}
