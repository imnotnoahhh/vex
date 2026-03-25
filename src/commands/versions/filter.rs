use super::RemoteFilter;
use crate::tools::Version;
use std::collections::HashMap;

pub(super) fn apply_filter(
    tool_name: &str,
    versions: Vec<Version>,
    filter: RemoteFilter,
) -> Vec<Version> {
    match filter {
        RemoteFilter::All => versions,
        RemoteFilter::Lts => {
            if tool_name == "python" {
                Vec::new()
            } else {
                versions
                    .into_iter()
                    .filter(|version| version.lts.is_some())
                    .collect()
            }
        }
        RemoteFilter::Major => newest_patch_per_major(versions),
        RemoteFilter::Latest => versions.into_iter().take(1).collect(),
    }
}

pub(super) fn remote_filter_name(filter: RemoteFilter) -> &'static str {
    match filter {
        RemoteFilter::All => "all",
        RemoteFilter::Lts => "lts",
        RemoteFilter::Major => "major",
        RemoteFilter::Latest => "latest",
    }
}

pub(super) fn is_version_outdated(version: &str, latest: &str) -> bool {
    let version_major = extract_major_version(version).parse::<i32>().unwrap_or(0);
    let latest_major = extract_major_version(latest).parse::<i32>().unwrap_or(0);
    version_major > 0 && latest_major > 0 && version_major < latest_major - 2
}

fn newest_patch_per_major(versions: Vec<Version>) -> Vec<Version> {
    let mut major_versions: HashMap<String, Vec<Version>> = HashMap::new();
    for version in versions {
        major_versions
            .entry(extract_major_version(&version.version))
            .or_default()
            .push(version);
    }

    let mut result: Vec<_> = major_versions
        .into_values()
        .filter_map(|group| {
            group
                .into_iter()
                .max_by_key(|version| version_sort_key(&version.version))
        })
        .collect();
    result.sort_by(|a, b| version_sort_key(&b.version).cmp(&version_sort_key(&a.version)));
    result
}

fn extract_major_version(version: &str) -> String {
    let version = version.strip_prefix('v').unwrap_or(version);
    version.split('.').next().unwrap_or("0").to_string()
}

fn version_sort_key(version: &str) -> Vec<u32> {
    version
        .trim_start_matches('v')
        .split('.')
        .filter_map(|segment| segment.parse().ok())
        .collect()
}
