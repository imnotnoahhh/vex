use super::releases::{create_github_client, fetch_text_with_retry};
use crate::error::{Result, VexError};
use std::collections::BTreeMap;

const PYTHON_STATUS_URL: &str = "https://devguide.python.org/versions/";

/// Python support status based on lifecycle
/// See: <https://devguide.python.org/versions/>
#[derive(Debug, Clone, PartialEq)]
pub(super) enum SupportStatus {
    Feature,
    Bugfix,
    Security,
    EndOfLife,
}

impl SupportStatus {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            SupportStatus::Feature => "feature",
            SupportStatus::Bugfix => "bugfix",
            SupportStatus::Security => "security",
            SupportStatus::EndOfLife => "end-of-life",
        }
    }

    pub(super) fn from_version(major_minor: &str) -> Self {
        match major_minor {
            "3.15" => SupportStatus::Feature,
            "3.14" | "3.13" => SupportStatus::Bugfix,
            "3.12" | "3.11" | "3.10" => SupportStatus::Security,
            _ => SupportStatus::EndOfLife,
        }
    }
}

pub(super) fn fetch_python_lifecycle_statuses() -> Result<BTreeMap<String, String>> {
    let client = create_github_client()?;
    let html = fetch_text_with_retry(&client, PYTHON_STATUS_URL)?;
    let statuses = parse_python_lifecycle_statuses(&html);
    if statuses.is_empty() {
        return Err(VexError::Parse(
            "Unable to parse Python lifecycle statuses from the official version page".to_string(),
        ));
    }
    Ok(statuses)
}

pub(super) fn fallback_python_lifecycle_statuses() -> BTreeMap<String, String> {
    let mut statuses = BTreeMap::new();
    for minor in ["3.15", "3.14", "3.13", "3.12", "3.11", "3.10"] {
        statuses.insert(
            minor.to_string(),
            SupportStatus::from_version(minor).as_str().to_string(),
        );
    }
    statuses
}

pub(super) fn parse_python_lifecycle_statuses(html: &str) -> BTreeMap<String, String> {
    let mut statuses = BTreeMap::new();
    let mut remaining = html;
    let mut seen_supported_rows = false;

    while let Some(tr_start) = remaining.find("<tr") {
        remaining = &remaining[tr_start..];
        let Some(row_end) = remaining.find("</tr>") else {
            break;
        };
        let row_html = &remaining[..row_end];
        let cells = parse_table_cells(row_html);
        if cells.len() >= 3 {
            let branch = cells[0].trim();
            let status = cells[2].trim().to_lowercase();
            if branch.starts_with("3.") {
                statuses.insert(branch.to_string(), status);
                seen_supported_rows = true;
            } else if seen_supported_rows && branch.is_empty() {
                break;
            }
        }
        remaining = &remaining[row_end + "</tr>".len()..];
    }

    statuses
}

fn strip_html_tags(text: &str) -> String {
    let mut out = String::new();
    let mut inside_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ").trim().to_string()
}

fn parse_table_cells(row_html: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut remaining = row_html;
    while let Some(td_start) = remaining.find("<td") {
        remaining = &remaining[td_start..];
        let Some(cell_start) = remaining.find('>') else {
            break;
        };
        remaining = &remaining[cell_start + 1..];
        let Some(cell_end) = remaining.find("</td>") else {
            break;
        };
        cells.push(strip_html_tags(&remaining[..cell_end]));
        remaining = &remaining[cell_end + "</td>".len()..];
    }
    cells
}
