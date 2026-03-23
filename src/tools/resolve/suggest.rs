use super::Version;
use crate::versioning::normalize_version;

pub(in crate::tools) fn generate_version_suggestions(
    requested: &str,
    available: &[Version],
) -> String {
    if available.is_empty() {
        return String::new();
    }

    let mut suggestions = Vec::new();
    let parts: Vec<&str> = requested.split('.').collect();
    let requested_major = parts.first().and_then(|s| s.parse::<u32>().ok());
    let requested_minor = parts.get(1).and_then(|s| s.parse::<u32>().ok());

    if let (Some(major), Some(minor)) = (requested_major, requested_minor) {
        if let Some(version) = latest_same_minor(available, major, minor) {
            suggestions.push(format!("  - {} (latest in {}.{}.x)", version, major, minor));
        }
    }

    if let Some(major) = requested_major {
        if let Some(version) = latest_same_major(available, major) {
            if !suggestions.iter().any(|entry| entry.contains(&version)) {
                suggestions.push(format!("  - {} (latest in {}.x)", version, major));
            }
        }
    }

    if let Some(major) = requested_major {
        for version in nearby_versions(available, major) {
            if !suggestions.iter().any(|entry| entry.contains(&version)) {
                suggestions.push(format!("  - {}", version));
            }
        }
    }

    if let Some(latest) = available.first() {
        let latest_version = normalize_version(&latest.version);
        if !suggestions
            .iter()
            .any(|entry| entry.contains(&latest_version))
        {
            suggestions.push(format!("  - {} (latest)", latest_version));
        }
    }

    if suggestions.is_empty() {
        String::new()
    } else {
        format!("\n\nDid you mean:\n{}", suggestions.join("\n"))
    }
}

fn latest_same_minor(available: &[Version], major: u32, minor: u32) -> Option<String> {
    available
        .iter()
        .filter(|version| version_major_minor(version) == Some((major, minor)))
        .max_by_key(|version| normalize_version(&version.version))
        .map(|version| normalize_version(&version.version))
}

fn latest_same_major(available: &[Version], major: u32) -> Option<String> {
    available
        .iter()
        .filter(|version| version_major(version) == Some(major))
        .max_by_key(|version| normalize_version(&version.version))
        .map(|version| normalize_version(&version.version))
}

fn nearby_versions(available: &[Version], major: u32) -> Vec<String> {
    available
        .iter()
        .filter_map(|version| {
            let normalized = normalize_version(&version.version);
            let version_major = normalized
                .split('.')
                .next()
                .and_then(|segment| segment.parse::<u32>().ok())?;
            if version_major.abs_diff(major) <= 2 && version_major != major {
                Some(normalized)
            } else {
                None
            }
        })
        .take(2)
        .collect()
}

fn version_major(version: &Version) -> Option<u32> {
    normalize_version(&version.version)
        .split('.')
        .next()
        .and_then(|segment| segment.parse::<u32>().ok())
}

fn version_major_minor(version: &Version) -> Option<(u32, u32)> {
    let normalized = normalize_version(&version.version);
    let mut parts = normalized.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    Some((major, minor))
}
