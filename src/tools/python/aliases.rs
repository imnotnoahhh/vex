use crate::tools::Version;

pub(super) fn resolve_alias_from_versions(versions: &[Version], alias: &str) -> Option<String> {
    match alias {
        "latest" | "stable" | "bugfix" => versions
            .iter()
            .find(|version| version.lts.as_deref() == Some("bugfix"))
            .map(|version| version.version.clone()),
        "security" => versions
            .iter()
            .find(|version| version.lts.as_deref() == Some("security"))
            .map(|version| version.version.clone()),
        _ => None,
    }
}
