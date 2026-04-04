use super::super::types::LifecycleWarning;

pub(super) fn build_suggestions(
    unused_version_count: usize,
    lifecycle_warnings: &[LifecycleWarning],
    issues: usize,
    has_home_hygiene_issue: bool,
    has_path_capture_issue: bool,
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

    if has_home_hygiene_issue {
        suggestions.push(
            "Run 'vex repair migrate-home' to preview safe home-directory migrations".to_string(),
        );
    }

    if has_path_capture_issue {
        suggestions.push("Reload your shell hook with 'eval \"$(vex env <shell>)\"' to refresh captured env vars and PATH".to_string());
    }

    suggestions
}
