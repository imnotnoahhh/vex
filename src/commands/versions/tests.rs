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
