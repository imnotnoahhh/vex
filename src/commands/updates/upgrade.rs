use super::{
    targets::{collect_targets, normalize_version, ManagedTarget},
    ManagedSource, UpgradeEntry, UpgradeReport,
};
use crate::config;
use crate::error::{Result, VexError};
use crate::installer;
use crate::switcher;
use crate::tools;
use crate::version_files;

pub(super) fn upgrade_all() -> Result<UpgradeReport> {
    let (scope, targets) = collect_targets(None)?;
    if targets.is_empty() {
        return Ok(UpgradeReport {
            scope,
            entries: Vec::new(),
        });
    }

    let mut entries = Vec::new();
    for target in targets {
        entries.push(upgrade_target(&target)?);
    }

    entries.sort_by(|a, b| a.tool.cmp(&b.tool));
    Ok(UpgradeReport { scope, entries })
}

pub(super) fn upgrade_one(tool_name: &str) -> Result<UpgradeReport> {
    let (scope, targets) = collect_targets(Some(tool_name))?;
    let target = targets
        .into_iter()
        .next()
        .ok_or_else(|| VexError::VersionNotFound {
            tool: tool_name.to_string(),
            version: "managed version".to_string(),
            suggestions: String::new(),
        })?;

    Ok(UpgradeReport {
        scope,
        entries: vec![upgrade_target(&target)?],
    })
}

fn upgrade_target(target: &ManagedTarget) -> Result<UpgradeEntry> {
    let tool = tools::get_tool(&target.tool)?;
    let latest = tools::resolve_fuzzy_version(tool.as_ref(), "latest")?;
    let previous_version = normalize_version(&target.version);
    let target_version = normalize_version(&latest);

    let status = if previous_version == target_version {
        "already_latest".to_string()
    } else {
        install_and_activate_target(tool.as_ref(), target, &target_version)?;
        "upgraded".to_string()
    };

    Ok(UpgradeEntry {
        tool: target.tool.clone(),
        previous_version,
        target_version,
        status,
        source: target.source,
        source_path: target
            .source_path
            .as_ref()
            .map(|path| path.display().to_string()),
    })
}

fn install_and_activate_target(
    tool: &dyn tools::Tool,
    target: &ManagedTarget,
    target_version: &str,
) -> Result<()> {
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let install_dir = vex_dir
        .join("toolchains")
        .join(&target.tool)
        .join(target_version);

    if !install_dir.exists() {
        installer::install(tool, target_version)?;
    }
    switcher::switch_version(tool, target_version)?;

    if matches!(
        target.source,
        ManagedSource::Project | ManagedSource::Global
    ) {
        if let Some(source_path) = &target.source_path {
            version_files::write_tool_version(source_path, &target.tool, target_version)?;
        }
    }

    Ok(())
}
