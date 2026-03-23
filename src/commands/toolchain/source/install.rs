use super::summary::{print_install_summary, InstallResult};
use crate::error::{Result, VexError};
use crate::installer;
use crate::paths::vex_dir;
use crate::resolver;
use crate::spec::parse_spec;
use crate::switcher;
use crate::team_config;
use crate::tools::{self, Tool};
use std::fs;

pub fn install_from_source(source: &str, offline: bool) -> Result<()> {
    let loaded = team_config::load_versions_from_source(source, &resolver::current_dir(), offline)?;
    if loaded.versions.is_empty() {
        println!("No versions found in {}", loaded.description);
        return Ok(());
    }

    let results = install_version_pairs(&loaded.versions, offline, false)?;
    print_install_summary(&results);
    Ok(())
}

pub fn install_specs(specs: &[String], no_switch: bool, force: bool, offline: bool) -> Result<()> {
    let vex = vex_dir()?;
    let mut results = Vec::new();

    for spec in specs {
        let (tool_name, version) = parse_spec(spec)?;
        if version.is_empty() {
            return Err(VexError::Parse(format!(
                "Version required for multi-spec install: {}",
                spec
            )));
        }

        let tool = match tools::get_tool(&tool_name) {
            Ok(tool) => tool,
            Err(error) => {
                results.push((tool_name.clone(), version.clone(), Err(error)));
                continue;
            }
        };

        let resolved = match tools::resolve_fuzzy_version(tool.as_ref(), &version) {
            Ok(version) => version,
            Err(error) => {
                results.push((tool_name.clone(), version.clone(), Err(error)));
                continue;
            }
        };

        let install_dir = vex.join("toolchains").join(&tool_name).join(&resolved);
        if install_dir.exists() && !force {
            results.push((tool_name.clone(), resolved.clone(), Ok(false)));
            continue;
        }
        if force && install_dir.exists() {
            fs::remove_dir_all(&install_dir)?;
        }

        let result = install_single(tool.as_ref(), &resolved, offline, !no_switch);
        results.push((tool_name.clone(), resolved, result));
    }

    print_install_summary(&results);

    let failed = results
        .iter()
        .filter(|(_, _, result)| result.is_err())
        .count();
    if failed > 0 {
        return Err(VexError::Config(format!(
            "{} installation(s) failed",
            failed
        )));
    }

    Ok(())
}

pub fn sync_from_source(source: &str, offline: bool) -> Result<()> {
    let loaded = team_config::load_versions_from_source(source, &resolver::current_dir(), offline)?;
    if loaded.versions.is_empty() {
        println!("No versions found in {}", loaded.description);
        return Ok(());
    }

    sync_versions(&loaded.versions, offline)
}

pub(in crate::commands::toolchain) fn sync_versions(
    versions: &[(String, String)],
    offline: bool,
) -> Result<()> {
    let results = install_version_pairs(versions, offline, true)?;
    print_install_summary(&results);
    Ok(())
}

fn install_version_pairs(
    versions: &[(String, String)],
    offline: bool,
    switch_after_install: bool,
) -> Result<Vec<InstallResult>> {
    let vex = vex_dir()?;
    let mut results = Vec::new();

    for (tool_name, version) in versions {
        let tool = match tools::get_tool(tool_name) {
            Ok(tool) => tool,
            Err(error) => {
                results.push((tool_name.clone(), version.clone(), Err(error)));
                continue;
            }
        };

        let install_dir = vex.join("toolchains").join(tool_name).join(version);
        if install_dir.exists() {
            results.push((tool_name.clone(), version.clone(), Ok(false)));
            continue;
        }

        let result = install_single(tool.as_ref(), version, offline, switch_after_install);
        results.push((tool_name.clone(), version.clone(), result));
    }

    Ok(results)
}

fn install_single(
    tool: &dyn Tool,
    version: &str,
    offline: bool,
    switch_after_install: bool,
) -> Result<bool> {
    installer::install_with_mode(tool, version, offline)?;
    if switch_after_install {
        let _ = switcher::switch_version(tool, version);
    }
    Ok(true)
}
