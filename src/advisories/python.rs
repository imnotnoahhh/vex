use super::{Advisory, AdvisoryStatus};

pub(super) fn python_advisory(version: &str) -> Advisory {
    let version = version.trim_start_matches('v');
    let parts: Vec<&str> = version.split('.').collect();
    let major = parts
        .first()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let minor = parts
        .get(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    if major == 2 {
        return Advisory::new(AdvisoryStatus::Eol)
            .with_message("python@2 is end-of-life".to_string())
            .with_recommendation("upgrade to python@3.12 or python@3.13".to_string());
    }

    match minor {
        0..=7 => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("python@3.{} is end-of-life", minor))
            .with_recommendation("upgrade to python@3.12 or python@3.13".to_string()),
        8 => Advisory::new(AdvisoryStatus::NearEol)
            .with_message("python@3.8 is near end-of-life".to_string())
            .with_recommendation("upgrade to python@3.12 or python@3.13".to_string()),
        9 | 10 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message(format!("python@3.{} is in security-only mode", minor))
            .with_recommendation("consider upgrading to python@3.12 or python@3.13".to_string()),
        11 => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message("python@3.11 is stable".to_string())
            .with_recommendation("consider upgrading to python@3.12 or python@3.13".to_string()),
        12 | 13 => Advisory::new(AdvisoryStatus::Current),
        _ => Advisory::new(AdvisoryStatus::Current),
    }
}
