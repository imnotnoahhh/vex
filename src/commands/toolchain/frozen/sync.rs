use super::super::source::sync_versions;
use super::lockfile_support::{load_lockfile_for_frozen, validate_lockfile_matches_versions};
use super::NO_VERSION_FILES_MESSAGE;
use crate::error::{Result, VexError};
use crate::resolver;

pub(super) fn from_current_context(offline: bool) -> Result<()> {
    let cwd = resolver::current_dir();
    let versions = resolver::resolve_versions(&cwd);

    if versions.is_empty() {
        println!("{}", NO_VERSION_FILES_MESSAGE);
        return Ok(());
    }

    let versions_vec = versions.into_iter().collect::<Vec<_>>();
    sync_versions(&versions_vec, offline)
}

pub(super) fn from_lockfile(offline: bool) -> Result<()> {
    let cwd = resolver::current_dir();
    let lockfile = load_lockfile_for_frozen(&cwd)?;
    let versions = resolver::resolve_versions(&cwd);

    if versions.is_empty() {
        return Err(VexError::Config(NO_VERSION_FILES_MESSAGE.to_string()));
    }

    validate_lockfile_matches_versions(&lockfile, &versions)?;

    let versions_vec = lockfile
        .tools
        .iter()
        .map(|(tool, entry)| (tool.clone(), entry.version.clone()))
        .collect::<Vec<_>>();
    sync_versions(&versions_vec, offline)
}
