use crate::error::Result;
use crate::http;
use crate::versioning::version_sort_key;
use std::cmp::Reverse;

const STABLE_ARCHIVE_URL: &str =
    "https://forge.rust-lang.org/infra/archive-stable-version-installers.html";
const HEADING_PREFIX: &str = "Stable (";

pub(super) fn fetch_archived_versions(target_triple: &str) -> Result<Vec<String>> {
    let content = http::get_text_in_current_context(
        STABLE_ARCHIVE_URL,
        concat!("vex/", env!("CARGO_PKG_VERSION")),
    )?;
    Ok(parse_archived_versions(&content, target_triple))
}

fn parse_archived_versions(content: &str, target_triple: &str) -> Vec<String> {
    let headings = extract_headings(content);
    let mut versions = Vec::new();

    for (index, (start, version)) in headings.iter().enumerate() {
        let end = headings
            .get(index + 1)
            .map(|(next_start, _)| *next_start)
            .unwrap_or(content.len());
        let block = &content[*start..end];

        if block.contains(target_triple) {
            versions.push(version.clone());
        }
    }

    versions.sort_by_key(|version| Reverse(version_sort_key(version)));
    versions.dedup();
    versions
}

fn extract_headings(content: &str) -> Vec<(usize, String)> {
    let mut headings = Vec::new();
    let mut offset = 0;

    while let Some(relative_start) = content[offset..].find(HEADING_PREFIX) {
        let start = offset + relative_start;
        let version_start = start + HEADING_PREFIX.len();
        let tail = &content[version_start..];
        let Some(relative_end) = tail.find(')') else {
            break;
        };

        let version = &tail[..relative_end];
        if is_stable_version(version) {
            headings.push((start, version.to_string()));
        }

        offset = version_start + relative_end + 1;
    }

    headings
}

fn is_stable_version(version: &str) -> bool {
    let mut segments = version.split('.');
    matches!(
        (segments.next(), segments.next(), segments.next(), segments.next()),
        (Some(major), Some(minor), Some(patch), None)
            if major.parse::<u32>().is_ok()
                && minor.parse::<u32>().is_ok()
                && patch.parse::<u32>().is_ok()
    )
}

#[cfg(test)]
mod tests {
    use super::parse_archived_versions;

    #[test]
    fn parse_archived_versions_filters_by_target_and_sorts_descending() {
        let content = r#"
            <h2>Stable (1.93.1)</h2>
            <table><tr><td>x86_64-apple-darwin</td></tr></table>
            <h2>Stable (1.93.0)</h2>
            <table><tr><td>aarch64-apple-darwin</td></tr></table>
            <h2>Stable (1.92.0)</h2>
            <table><tr><td>x86_64-apple-darwin</td></tr></table>
        "#;

        let versions = parse_archived_versions(content, "x86_64-apple-darwin");
        assert_eq!(versions, vec!["1.93.1", "1.92.0"]);
    }

    #[test]
    fn parse_archived_versions_keeps_arm64_supported_releases_only() {
        let content = r#"
            <h2>Stable (1.40.0)</h2>
            <table><tr><td>x86_64-apple-darwin</td></tr></table>
            <h2>Stable (1.39.0)</h2>
            <table><tr><td>x86_64-apple-darwin</td></tr></table>
            <h2>Stable (1.38.0)</h2>
            <table><tr><td>aarch64-apple-darwin</td></tr></table>
        "#;

        let versions = parse_archived_versions(content, "aarch64-apple-darwin");
        assert_eq!(versions, vec!["1.38.0"]);
    }

    #[test]
    fn parse_archived_versions_ignores_non_version_headings() {
        let content = r#"
            <h2>Stable releases</h2>
            <h2>Stable (latest)</h2>
            <h2>Stable (1.93.1)</h2>
            <table><tr><td>x86_64-apple-darwin</td></tr></table>
        "#;

        let versions = parse_archived_versions(content, "x86_64-apple-darwin");
        assert_eq!(versions, vec!["1.93.1"]);
    }
}
