use crate::commands::doctor::types::DoctorReport;
use crate::ui;

pub(super) fn render_summary(report: &DoctorReport) {
    let mut summary = ui::Summary::new();
    if report.issues == 0 && report.warnings == 0 {
        summary = summary.success("All checks passed!".to_string());
    } else {
        if report.issues > 0 {
            summary = summary.error(format!("{} issue(s) found", report.issues));
        }
        if report.warnings > 0 {
            summary = summary.warning(format!("{} warning(s)", report.warnings));
        }
    }

    for suggestion in &report.suggestions {
        summary = summary.info(suggestion.clone());
    }

    summary.render();
}
