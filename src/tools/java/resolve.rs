use super::api::{available_versions, lts_versions, AvailableReleases, FALLBACK_LTS_VERSIONS};
use super::JavaTool;
use crate::error::Result;
use crate::tools::{Arch, Tool, Version};

pub(super) fn build_remote_versions(releases: &AvailableReleases) -> Vec<Version> {
    let lts_versions_list = lts_versions(releases);
    let mut versions = available_versions(releases)
        .into_iter()
        .map(|version| Version {
            version: version.to_string(),
            lts: if lts_versions_list.contains(&version) {
                Some("LTS".to_string())
            } else {
                None
            },
        })
        .collect::<Vec<_>>();

    versions.reverse();
    versions
}

pub(super) fn resolve_alias_version(
    tool: &JavaTool,
    alias: &str,
    versions: &[Version],
) -> Result<Option<String>> {
    match alias {
        "latest" => Ok(versions.first().map(|version| version.version.clone())),
        "lts" => {
            if let Some(version) = versions
                .iter()
                .find(|version| version.lts.is_some())
                .map(|version| version.version.clone())
            {
                return Ok(Some(version));
            }

            let arch = Arch::detect()?;
            for candidate in FALLBACK_LTS_VERSIONS {
                if tool.download_url(&candidate.to_string(), arch).is_ok() {
                    return Ok(Some(candidate.to_string()));
                }
            }

            Ok(None)
        }
        _ => Ok(None),
    }
}
