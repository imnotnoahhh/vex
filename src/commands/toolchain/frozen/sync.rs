use super::super::source::sync_versions;
use super::lockfile_support::{
    load_lockfile_for_frozen, locked_tools, validate_lockfile_matches_versions,
    verify_installed_checksum, LockedTool,
};
use super::NO_VERSION_FILES_MESSAGE;
use crate::error::{Result, VexError};
use crate::installer;
use crate::paths::vex_dir;
use crate::requested_versions;
use crate::resolver;
use crate::switcher;
use crate::tools;

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

    let locked = locked_tools(&lockfile)?;
    sync_locked_versions(&locked, offline)
}

fn sync_locked_versions(locked_versions: &[LockedTool], offline: bool) -> Result<()> {
    let vex = vex_dir()?;

    for locked in locked_versions {
        let tool = match tools::get_tool(&locked.tool) {
            Ok(tool) => tool,
            Err(_) => {
                eprintln!("vex: skipping unsupported tool '{}'", locked.tool);
                continue;
            }
        };

        let installed =
            requested_versions::resolve_installed_version(&vex, &locked.tool, &locked.version)?;
        if let Some(installed) = installed {
            verify_installed_checksum(&vex, &locked.tool, &installed, &locked.sha256)?;
            switcher::switch_version(tool.as_ref(), &installed)?;
            println!("{}@{} already installed, verified.", locked.tool, installed);
            continue;
        }

        let version_dir = vex
            .join("toolchains")
            .join(&locked.tool)
            .join(&locked.version);
        if version_dir.exists() {
            verify_installed_checksum(&vex, &locked.tool, &locked.version, &locked.sha256)?;
        } else {
            installer::install_with_mode_and_checksum(
                tool.as_ref(),
                &locked.version,
                offline,
                Some(&locked.sha256),
            )?;
        }
        switcher::switch_version(tool.as_ref(), &locked.version)?;
    }

    Ok(())
}
