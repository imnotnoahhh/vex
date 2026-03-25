use super::{Advisory, AdvisoryStatus};

pub(super) fn java_advisory(version: &str) -> Advisory {
    let version = version.trim_start_matches('v');
    let major = version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    match major {
        0..=7 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("java@{} is end-of-life", major))
            .with_recommendation("upgrade to java@21 (current LTS)".to_string()),
        8 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("java@8 is very old".to_string())
            .with_recommendation("upgrade to java@21 (current LTS)".to_string()),
        9 | 10 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("java@{} is end-of-life", major))
            .with_recommendation("upgrade to java@21 (current LTS)".to_string()),
        11 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("java@11 is an older LTS".to_string())
            .with_recommendation("consider upgrading to java@21 (current LTS)".to_string()),
        12..=16 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("java@{} is end-of-life", major))
            .with_recommendation("upgrade to java@21 (current LTS)".to_string()),
        17 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("java@17 is an older LTS".to_string())
            .with_recommendation("consider upgrading to java@21 (current LTS)".to_string()),
        18..=20 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("java@{} is end-of-life", major))
            .with_recommendation("upgrade to java@21 (current LTS)".to_string()),
        21..=23 => Advisory::new(AdvisoryStatus::Current),
        _ => Advisory::new(AdvisoryStatus::Current),
    }
}
