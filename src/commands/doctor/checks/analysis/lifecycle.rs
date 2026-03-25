use crate::advisories::{self, AdvisoryStatus};
use crate::commands::doctor::types::LifecycleWarning;
use crate::error::Result;
use std::fs;
use std::path::Path;

pub(super) fn collect_lifecycle_warnings(vex_dir: &Path) -> Result<Vec<LifecycleWarning>> {
    let toolchains_dir = vex_dir.join("toolchains");
    if !toolchains_dir.exists() {
        return Ok(Vec::new());
    }

    let mut warnings = Vec::new();
    for tool_name in &["node", "java", "python"] {
        let tool_dir = toolchains_dir.join(tool_name);
        if !tool_dir.exists() {
            continue;
        }

        for version_entry in fs::read_dir(&tool_dir)?.filter_map(|entry| entry.ok()) {
            if !version_entry
                .file_type()
                .ok()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
            {
                continue;
            }

            let version = version_entry.file_name().to_string_lossy().to_string();
            let advisory = advisories::get_advisory(tool_name, &version);

            if advisory.is_warning() {
                let status = match advisory.status {
                    AdvisoryStatus::Eol => "eol",
                    AdvisoryStatus::NearEol => "near_eol",
                    AdvisoryStatus::LtsAvailable => "lts_available",
                    AdvisoryStatus::SecurityUpdateAvailable => "security_update_available",
                    _ => continue,
                };

                let message = advisory
                    .message
                    .or(advisory.recommendation)
                    .unwrap_or_else(|| format!("{} {} has lifecycle concerns", tool_name, version));

                warnings.push(LifecycleWarning {
                    tool: tool_name.to_string(),
                    version,
                    status: status.to_string(),
                    message,
                });
            }
        }
    }

    warnings.sort_by(|left, right| {
        left.tool
            .cmp(&right.tool)
            .then(left.version.cmp(&right.version))
    });
    Ok(warnings)
}
