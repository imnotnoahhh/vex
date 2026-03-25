mod api;
mod asset;
mod version;

pub(in crate::updater) use api::{fetch_latest_release, GithubAsset, GithubRelease};
pub(in crate::updater) use asset::{asset_name, select_release_asset};
pub(in crate::updater) use version::{is_newer, strip_v, version_tuple};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_v() {
        assert_eq!(strip_v("v0.1.7"), "0.1.7");
        assert_eq!(strip_v("0.1.7"), "0.1.7");
        assert_eq!(strip_v("v1.2.3"), "1.2.3");
        assert_eq!(strip_v("1.2.3"), "1.2.3");
        assert_eq!(strip_v("v0.0.1"), "0.0.1");
        assert_eq!(strip_v(""), "");
        assert_eq!(strip_v("v"), "");
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.1.6", "0.1.7"));
        assert!(is_newer("0.1.6", "0.2.0"));
        assert!(is_newer("0.1.6", "1.0.0"));
        assert!(!is_newer("0.1.7", "0.1.7"));
        assert!(!is_newer("0.1.7", "0.1.6"));
        assert!(is_newer("1.0.0", "2.0.0"));
        assert!(!is_newer("2.0.0", "1.0.0"));
        assert!(is_newer("1.1.0", "1.2.0"));
        assert!(!is_newer("1.2.0", "1.1.0"));
        assert!(is_newer("1.0.1", "1.0.2"));
        assert!(!is_newer("1.0.2", "1.0.1"));
        assert!(is_newer("1.9.9", "1.10.0"));
        assert!(is_newer("1.99.99", "2.0.0"));
    }

    #[test]
    fn test_version_tuple_for_repair_threshold() {
        assert!(version_tuple("1.0.0") < (1, 1, 0));
        assert!(version_tuple("1.0.1") < (1, 1, 0));
        assert!(version_tuple("0.9.9") < (1, 1, 0));
        assert!(version_tuple("1.1.0") >= (1, 1, 0));
        assert!(version_tuple("1.1.1") >= (1, 1, 0));
        assert!(version_tuple("2.0.0") >= (1, 1, 0));
    }

    #[test]
    fn test_asset_name_returns_supported_format() {
        let name = asset_name();
        assert!(name.is_some());
        let name = name.unwrap();
        assert!(name.contains("darwin") || name.contains("linux"));
        assert!(name.contains("aarch64") || name.contains("x86_64"));
    }
}
