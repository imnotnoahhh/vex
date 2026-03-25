use super::super::super::types::{CheckStatus, DoctorCheck};
use std::process::Command;

pub(super) fn push_network_check(checks: &mut Vec<DoctorCheck>, warnings: &mut usize) {
    let network_check = match Command::new("ping")
        .args(["-c", "1", "-W", "2", "nodejs.org"])
        .output()
    {
        Ok(output) if output.status.success() => DoctorCheck {
            id: "network".to_string(),
            status: CheckStatus::Ok,
            summary: "basic network connectivity is available".to_string(),
            details: Vec::new(),
        },
        _ => {
            *warnings += 1;
            DoctorCheck {
                id: "network".to_string(),
                status: CheckStatus::Warn,
                summary: "nodejs.org was unreachable during the health check".to_string(),
                details: vec!["Check your internet connection or firewall settings".to_string()],
            }
        }
    };
    checks.push(network_check);
}
