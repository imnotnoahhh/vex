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
pub struct ToolDiskUsage {
    pub tool: String,
    pub version_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct UnusedVersion {
    pub tool: String,
    pub version: String,
    pub bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct LifecycleWarning {
    pub tool: String,
    pub version: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub root: String,
    pub issues: usize,
    pub warnings: usize,
    pub checks: Vec<DoctorCheck>,
    pub disk_usage: Vec<ToolDiskUsage>,
    pub unused_versions: Vec<UnusedVersion>,
    pub lifecycle_warnings: Vec<LifecycleWarning>,
    pub total_disk_bytes: u64,
    pub reclaimable_bytes: u64,
    pub suggestions: Vec<String>,
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
