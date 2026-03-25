use super::{Advisory, AdvisoryStatus};

pub(super) fn node_advisory(version: &str) -> Advisory {
    let version = version.trim_start_matches('v');
    let major = version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    match major {
        0..=15 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("node@{} is end-of-life", major))
            .with_recommendation("upgrade to node@22 (current LTS)".to_string()),
        16 => Advisory::new(AdvisoryStatus::Eol)
            .with_message("node@16 is end-of-life".to_string())
            .with_recommendation("upgrade to node@22 (current LTS)".to_string()),
        17 => Advisory::new(AdvisoryStatus::Eol)
            .with_message("node@17 is end-of-life".to_string())
            .with_recommendation("upgrade to node@22 (current LTS)".to_string()),
        19 => Advisory::new(AdvisoryStatus::Eol)
            .with_message("node@19 is end-of-life".to_string())
            .with_recommendation("upgrade to node@22 (current LTS)".to_string()),
        21 => Advisory::new(AdvisoryStatus::Eol)
            .with_message("node@21 is end-of-life".to_string())
            .with_recommendation("upgrade to node@22 (current LTS)".to_string()),
        18 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("node@18 is in maintenance mode".to_string())
            .with_recommendation("consider upgrading to node@22 (current LTS)".to_string()),
        20 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("node@20 is in maintenance mode".to_string())
            .with_recommendation("consider upgrading to node@22 (current LTS)".to_string()),
        22 | 23 => Advisory::new(AdvisoryStatus::Current),
        _ => Advisory::new(AdvisoryStatus::Current),
    }
}
