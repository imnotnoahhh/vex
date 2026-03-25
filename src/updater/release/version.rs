pub(in crate::updater) fn strip_v(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

pub(in crate::updater) fn is_newer(local: &str, remote: &str) -> bool {
    version_tuple(remote) > version_tuple(local)
}

pub(in crate::updater) fn version_tuple(version: &str) -> (u64, u64, u64) {
    let parts: Vec<u64> = version
        .split('.')
        .filter_map(|part| part.parse().ok())
        .collect();
    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}
