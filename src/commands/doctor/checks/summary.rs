use super::super::types::LifecycleWarning;

pub(super) fn build_suggestions(
    unused_version_count: usize,
    lifecycle_warnings: &[LifecycleWarning],
    issues: usize,
) -> Vec<String> {
    let mut suggestions = Vec::new();

    if unused_version_count > 0 {
        suggestions.push(format!(
            "Run 'vex prune --dry-run' to see {} unused version(s) that can be removed",
            unused_version_count
        ));
    }

    if !lifecycle_warnings.is_empty() {
        let outdated_count = lifecycle_warnings
            .iter()
            .filter(|warning| warning.status == "outdated" || warning.status == "near_eol")
            .count();
        if outdated_count > 0 {
            suggestions.push(format!(
                "Run 'vex outdated' to check for updates to {} tool(s)",
                outdated_count
            ));
        }
    }

    if issues > 0 {
        suggestions.push("Run 'vex init' to fix structural issues".to_string());
    }

    suggestions
}
