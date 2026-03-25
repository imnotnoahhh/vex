mod checks;
mod sections;
mod summary;

use super::types::DoctorReport;
use crate::ui;

pub(super) fn render_text(report: &DoctorReport) {
    ui::header("vex doctor - Health Check");
    checks::render_checks(&report.checks);
    println!();
    sections::render_sections(report);
    summary::render_summary(report);
}
