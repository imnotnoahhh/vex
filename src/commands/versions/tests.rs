use super::filter::apply_filter;
use super::*;
use crate::tools::Version;

#[test]
fn test_major_filter_keeps_newest_patch_per_major() {
    let filtered = apply_filter(
        "node",
        vec![
            Version {
                version: "20.10.0".to_string(),
                lts: None,
            },
            Version {
                version: "20.9.0".to_string(),
                lts: None,
            },
            Version {
                version: "19.8.1".to_string(),
                lts: None,
            },
            Version {
                version: "19.8.0".to_string(),
                lts: None,
            },
        ],
        RemoteFilter::Major,
    );

    let versions = filtered
        .into_iter()
        .map(|version| version.version)
        .collect::<Vec<_>>();
    assert_eq!(versions, vec!["20.10.0", "19.8.1"]);
}

#[test]
fn test_python_latest_filter_skips_feature_prereleases() {
    let filtered = apply_filter(
        "python",
        vec![
            Version {
                version: "3.15.0a8".to_string(),
                lts: Some("feature".to_string()),
            },
            Version {
                version: "3.14.4".to_string(),
                lts: Some("bugfix".to_string()),
            },
            Version {
                version: "3.13.12".to_string(),
                lts: Some("security".to_string()),
            },
        ],
        RemoteFilter::Latest,
    );

    let versions = filtered
        .into_iter()
        .map(|version| version.version)
        .collect::<Vec<_>>();
    assert_eq!(versions, vec!["3.14.4"]);
}

#[test]
fn test_python_major_filter_prefers_supported_stable_versions() {
    let filtered = apply_filter(
        "python",
        vec![
            Version {
                version: "3.15.0a8".to_string(),
                lts: Some("feature".to_string()),
            },
            Version {
                version: "3.14.4".to_string(),
                lts: Some("bugfix".to_string()),
            },
            Version {
                version: "3.13.12".to_string(),
                lts: Some("security".to_string()),
            },
        ],
        RemoteFilter::Major,
    );

    let versions = filtered
        .into_iter()
        .map(|version| version.version)
        .collect::<Vec<_>>();
    assert_eq!(versions, vec!["3.14.4"]);
}
