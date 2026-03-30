use super::{Advisory, AdvisoryStatus};
use chrono::{NaiveDate, Utc};

#[derive(Clone, Copy)]
struct NodeLine {
    major: u32,
    start: NaiveDate,
    lts: Option<NaiveDate>,
    maintenance: Option<NaiveDate>,
    end: NaiveDate,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NodePhase {
    Current,
    ActiveLts,
    MaintenanceLts,
    Eol,
}

const NODE_LINES: &[NodeLine] = &[
    node_line(
        16,
        "2021-04-20",
        Some("2021-10-26"),
        Some("2022-10-18"),
        "2023-09-11",
    ),
    node_line(17, "2021-10-19", None, None, "2022-06-01"),
    node_line(
        18,
        "2022-04-19",
        Some("2022-10-25"),
        Some("2023-10-18"),
        "2025-04-30",
    ),
    node_line(19, "2022-10-18", None, None, "2023-06-01"),
    node_line(
        20,
        "2023-04-18",
        Some("2023-10-24"),
        Some("2024-10-22"),
        "2026-04-30",
    ),
    node_line(21, "2023-10-17", None, None, "2024-06-01"),
    node_line(
        22,
        "2024-04-24",
        Some("2024-10-29"),
        Some("2025-10-21"),
        "2027-04-30",
    ),
    node_line(23, "2024-10-16", None, None, "2025-06-01"),
    node_line(
        24,
        "2025-05-06",
        Some("2025-10-28"),
        Some("2026-10-20"),
        "2028-04-30",
    ),
    node_line(25, "2025-10-15", None, None, "2026-06-01"),
    node_line(
        26,
        "2026-04-22",
        Some("2026-10-28"),
        Some("2027-10-20"),
        "2029-04-30",
    ),
];

const fn node_line(
    major: u32,
    start: &str,
    lts: Option<&str>,
    maintenance: Option<&str>,
    end: &str,
) -> NodeLine {
    NodeLine {
        major,
        start: parse_date_const(start),
        lts: match lts {
            Some(date) => Some(parse_date_const(date)),
            None => None,
        },
        maintenance: match maintenance {
            Some(date) => Some(parse_date_const(date)),
            None => None,
        },
        end: parse_date_const(end),
    }
}

const fn parse_date_const(date: &str) -> NaiveDate {
    let bytes = date.as_bytes();
    let year = ((bytes[0] - b'0') as i32) * 1000
        + ((bytes[1] - b'0') as i32) * 100
        + ((bytes[2] - b'0') as i32) * 10
        + (bytes[3] - b'0') as i32;
    let month = ((bytes[5] - b'0') as u32) * 10 + (bytes[6] - b'0') as u32;
    let day = ((bytes[8] - b'0') as u32) * 10 + (bytes[9] - b'0') as u32;

    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(parsed) => parsed,
        None => panic!("invalid date"),
    }
}

pub(super) fn node_advisory(version: &str) -> Advisory {
    node_advisory_at(version, Utc::now().date_naive())
}

pub(super) fn node_advisory_at(version: &str, today: NaiveDate) -> Advisory {
    let version = version.trim_start_matches('v');
    let major = version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let Some(line) = NODE_LINES.iter().find(|line| line.major == major).copied() else {
        return if major < NODE_LINES[0].major {
            Advisory::new(AdvisoryStatus::Eol)
                .with_message(format!("node@{} is end-of-life", major))
                .with_recommendation(recommendation(today, "upgrade to"))
        } else {
            Advisory::new(AdvisoryStatus::Current)
        };
    };

    match phase_for(line, today) {
        NodePhase::Eol => Advisory::new(AdvisoryStatus::Eol)
            .with_message(format!("node@{} is end-of-life", major))
            .with_recommendation(recommendation(today, "upgrade to")),
        NodePhase::MaintenanceLts => Advisory::new(AdvisoryStatus::LtsAvailable)
            .with_message(format!("node@{} is in maintenance mode", major))
            .with_recommendation(recommendation(today, "consider upgrading to")),
        NodePhase::ActiveLts | NodePhase::Current => Advisory::new(AdvisoryStatus::Current),
    }
}

fn phase_for(line: NodeLine, today: NaiveDate) -> NodePhase {
    if today >= line.end {
        return NodePhase::Eol;
    }

    if let Some(maintenance) = line.maintenance {
        if today >= maintenance {
            return NodePhase::MaintenanceLts;
        }
    }

    if let Some(lts) = line.lts {
        if today >= lts {
            return NodePhase::ActiveLts;
        }
    }

    if today >= line.start {
        NodePhase::Current
    } else {
        NodePhase::Current
    }
}

fn recommendation(today: NaiveDate, prefix: &str) -> String {
    let Some((major, label)) = recommended_target(today) else {
        return "upgrade to the latest supported Node.js release".to_string();
    };

    format!("{prefix} node@{major} ({label})")
}

fn recommended_target(today: NaiveDate) -> Option<(u32, &'static str)> {
    if let Some(line) = NODE_LINES
        .iter()
        .copied()
        .filter(|line| phase_for(*line, today) == NodePhase::ActiveLts)
        .max_by_key(|line| line.major)
    {
        return Some((line.major, "current LTS"));
    }

    NODE_LINES
        .iter()
        .copied()
        .filter(|line| phase_for(*line, today) == NodePhase::Current)
        .max_by_key(|line| line.major)
        .map(|line| (line.major, "current release"))
}
