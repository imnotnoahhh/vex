use crate::config;
use crate::error::Result;
use crate::paths::vex_dir;
use crate::resolver;
use crate::switcher;
use crate::tools;
use std::fs;

mod frozen;
mod lockfile_cmd;
mod source;

pub use frozen::{install_from_version_files_with_frozen, sync_from_current_context_with_frozen};
pub use lockfile_cmd::generate_lockfile;
pub use source::{install_from_source, install_specs, sync_from_source};

pub fn auto_switch() -> Result<()> {
    if !config::auto_switch()? {
        return Ok(());
    }

    if let Some(project_config) =
        crate::project::load_nearest_project_config(&resolver::current_dir())?
    {
        if project_config.config.behavior.auto_switch == Some(false) {
            return Ok(());
        }
    }

    let cwd = resolver::current_dir();
    let versions = resolver::resolve_versions(&cwd);

    if versions.is_empty() {
        return Ok(());
    }

    let vex = vex_dir()?;

    for (tool_name, version) in &versions {
        let tool = match tools::get_tool(tool_name) {
            Ok(tool) => tool,
            Err(_) => continue,
        };

        let version_dir = vex.join("toolchains").join(tool_name).join(version);
        if !version_dir.exists() {
            eprintln!(
                "vex: {}@{} not installed. Run 'vex install' to install.",
                tool_name, version
            );
            continue;
        }

        let current_link = vex.join("current").join(tool_name);
        if current_link.exists() {
            if let Ok(target) = fs::read_link(&current_link) {
                if let Some(current_ver) = target.file_name() {
                    if current_ver.to_string_lossy() == version.as_str() {
                        continue;
                    }
                }
            }
        }

        switcher::switch_version(tool.as_ref(), version)?;
    }

    Ok(())
}
