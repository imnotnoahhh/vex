use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Serialize)]
pub struct DoctorCheck {
    pub id: String,
    pub status: CheckStatus,
    pub summary: String,
    pub details: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub root: String,
    pub issues: usize,
    pub warnings: usize,
    pub checks: Vec<DoctorCheck>,
}

pub(super) fn push_check(
    checks: &mut Vec<DoctorCheck>,
    id: &str,
    status: CheckStatus,
    summary: &str,
    details: Vec<String>,
) {
    checks.push(DoctorCheck {
        id: id.to_string(),
        status,
        summary: summary.to_string(),
        details,
    });
}
