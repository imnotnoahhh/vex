use super::{targets::collect_targets, targets::normalize_version, OutdatedEntry, OutdatedReport};
use crate::advisories::{self, AdvisoryStatus};
use crate::error::Result;
use crate::tools;

pub(super) fn collect_outdated(tool: Option<&str>) -> Result<OutdatedReport> {
    let (scope, targets) = collect_targets(tool)?;
    let mut entries = Vec::new();

    for target in targets {
        let tool_impl = tools::get_tool(&target.tool)?;
        let latest_version = tools::resolve_fuzzy_version(tool_impl.as_ref(), "latest")?;
        let advisory = advisories::get_advisory(&target.tool, &target.version);
        let (advisory_status, advisory_message, advisory_recommendation) =
            if advisory.status != AdvisoryStatus::Unknown {
                (
                    Some(format!("{:?}", advisory.status).to_lowercase()),
                    advisory.message,
                    advisory.recommendation,
                )
            } else {
                (None, None, None)
            };

        entries.push(OutdatedEntry {
            tool: target.tool,
            current_version: normalize_version(&target.version),
            latest_version: normalize_version(&latest_version),
            status: if normalize_version(&target.version) == normalize_version(&latest_version) {
                "up_to_date"
            } else {
                "outdated"
            }
            .to_string(),
            source: target.source,
            source_path: target.source_path.map(|path| path.display().to_string()),
            advisory_status,
            advisory_message,
            advisory_recommendation,
        });
    }

    entries.sort_by(|a, b| a.tool.cmp(&b.tool));
    Ok(OutdatedReport { scope, entries })
}
