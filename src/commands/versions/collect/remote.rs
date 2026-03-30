use super::state::{current_version_for_tool, version_matches_current};
use crate::cache;
use crate::commands::versions::{
    filter::{apply_filter, is_version_outdated, remote_filter_name},
    RemoteFilter, RemoteVersionEntry, RemoteVersionsReport,
};
use crate::config;
use crate::error::{Result, VexError};
use crate::tools::{self, Tool, Version};
use crate::versioning::normalize_version;
use indicatif::{ProgressBar, ProgressStyle};

pub(super) fn collect_remote_versions(
    tool_name: &str,
    filter: RemoteFilter,
    use_cache: bool,
    offline: bool,
    show_spinner: bool,
) -> Result<RemoteVersionsReport> {
    let tool = tools::get_tool(tool_name)?;
    let mut versions = if show_spinner {
        fetch_with_spinner(tool_name, tool.as_ref(), use_cache, offline)?
    } else {
        fetch_versions_cached(tool.as_ref(), use_cache, offline)?
    };

    let current_version = current_version_for_tool(tool_name);
    versions = apply_filter(tool_name, versions, filter);

    let latest_version = versions.first().map(|version| version.version.clone());
    let versions = versions
        .into_iter()
        .map(|version| RemoteVersionEntry {
            version: normalize_version(&version.version),
            label: version.lts.clone(),
            is_current: version_matches_current(current_version.as_deref(), &version.version),
            is_outdated: latest_version
                .as_ref()
                .map(|latest| is_version_outdated(&version.version, latest))
                .unwrap_or(false),
        })
        .collect::<Vec<_>>();

    Ok(RemoteVersionsReport {
        tool: tool_name.to_string(),
        filter: remote_filter_name(filter).to_string(),
        total: versions.len(),
        current_version,
        versions,
    })
}

fn fetch_with_spinner(
    tool_name: &str,
    tool: &dyn Tool,
    use_cache: bool,
    offline: bool,
) -> Result<Vec<Version>> {
    if crate::logging::diagnostics_enabled() {
        return fetch_versions_cached(tool, use_cache, offline);
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Fetching available versions of {}...", tool_name));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    let versions = fetch_versions_cached(tool, use_cache, offline)?;
    spinner.finish_and_clear();
    Ok(versions)
}

fn fetch_versions_cached(tool: &dyn Tool, use_cache: bool, offline: bool) -> Result<Vec<Version>> {
    let vex = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let remote_cache = cache::RemoteCache::new(&vex);
    let ttl = config::cache_ttl()?.as_secs();

    if use_cache || offline {
        if let Some(cached) = remote_cache.get_cached_versions(tool.name(), ttl) {
            return Ok(cached);
        }
    }

    if offline {
        return Err(VexError::OfflineModeError(format!(
            "No cached version data available for {} in offline mode",
            tool.name()
        )));
    }

    let versions = tool.list_remote()?;
    remote_cache.set_cached_versions(tool.name(), &versions);
    Ok(versions)
}
