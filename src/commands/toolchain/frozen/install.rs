use super::lockfile_support::{load_lockfile_for_frozen, validate_lockfile_matches_versions};
use super::NO_VERSION_FILES_MESSAGE;
use crate::error::Result;
use crate::installer;
use crate::paths::vex_dir;
use crate::requested_versions;
use crate::resolver;
use crate::switcher;
use crate::tools;

pub(super) fn from_version_files(offline: bool) -> Result<()> {
    let cwd = resolver::current_dir();
    let versions = resolver::resolve_versions(&cwd);

    if versions.is_empty() {
        println!("{}", NO_VERSION_FILES_MESSAGE);
        return Ok(());
    }

    let requested = versions.into_iter().collect::<Vec<_>>();
    install_requested_versions(&requested, offline)
}

pub(super) fn from_lockfile(offline: bool) -> Result<()> {
    let cwd = resolver::current_dir();
    let lockfile = load_lockfile_for_frozen(&cwd)?;
    let versions = resolver::resolve_versions(&cwd);
    validate_lockfile_matches_versions(&lockfile, &versions)?;

    let requested = lockfile
        .tools
        .iter()
        .map(|(tool, entry)| (tool.clone(), entry.version.clone()))
        .collect::<Vec<_>>();
    install_requested_versions(&requested, offline)
}

fn install_requested_versions(requested: &[(String, String)], offline: bool) -> Result<()> {
    let vex = vex_dir()?;

    for (tool_name, version) in requested {
        let tool = match tools::get_tool(tool_name) {
            Ok(tool) => tool,
            Err(_) => {
                eprintln!("vex: skipping unsupported tool '{}'", tool_name);
                continue;
            }
        };

        if let Some(installed) =
            requested_versions::resolve_installed_version(&vex, tool_name, version)?
        {
            println!("{}@{} already installed, skipping.", tool_name, installed);
            continue;
        }

        let resolved = requested_versions::resolve_for_install(tool.as_ref(), version)?;
        let version_dir = vex.join("toolchains").join(tool_name).join(&resolved);
        if version_dir.exists() {
            println!("{}@{} already installed, skipping.", tool_name, resolved);
            continue;
        }

        installer::install_with_mode(tool.as_ref(), &resolved, offline)?;
        switcher::switch_version(tool.as_ref(), &resolved)?;
    }

    Ok(())
}
