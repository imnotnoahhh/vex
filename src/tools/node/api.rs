use crate::error::Result;
use crate::http;
use crate::tools::Version;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(super) struct NodeRelease {
    version: String,
    lts: serde_json::Value,
}

pub(super) fn fetch_releases() -> Result<Vec<NodeRelease>> {
    http::get_json_in_current_context(
        "https://nodejs.org/dist/index.json",
        concat!("vex/", env!("CARGO_PKG_VERSION")),
    )
}

pub(super) fn version_from_release(release: NodeRelease) -> Version {
    Version {
        version: release.version,
        lts: match release.lts {
            serde_json::Value::String(value) => Some(value),
            _ => None,
        },
    }
}

pub(super) fn resolve_alias_from_versions(versions: &[Version], alias: &str) -> Option<String> {
    match alias {
        "latest" => versions
            .first()
            .map(|version| unprefixed_version(&version.version)),
        "lts" => versions
            .iter()
            .find(|version| version.lts.is_some())
            .map(|version| unprefixed_version(&version.version)),
        _ if alias.starts_with("lts-") => {
            let codename = alias.strip_prefix("lts-")?.to_lowercase();
            versions
                .iter()
                .find(|version| {
                    version
                        .lts
                        .as_ref()
                        .map(|lts| lts.to_lowercase() == codename)
                        .unwrap_or(false)
                })
                .map(|version| unprefixed_version(&version.version))
        }
        _ => None,
    }
}

fn unprefixed_version(version: &str) -> String {
    version.strip_prefix('v').unwrap_or(version).to_string()
}
