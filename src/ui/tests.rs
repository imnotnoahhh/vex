use super::*;

#[test]
fn test_ui_context_creation() {
    let ctx = UiContext::new();
    let _ = ctx.interactive;

    let ctx = UiContext::non_interactive();
    assert!(!ctx.interactive);
}

#[test]
fn test_table_empty() {
    let table = Table::new();
    table.render();
}

#[test]
fn test_table_with_rows() {
    let table = Table::new()
        .headers(vec!["Tool".to_string(), "Version".to_string()])
        .row(vec!["node".to_string(), "20.0.0".to_string()])
        .row(vec!["go".to_string(), "1.21.0".to_string()]);

    table.render();
}

#[test]
fn test_summary_empty() {
    let summary = Summary::new();
    summary.render();
}

#[test]
fn test_summary_with_items() {
    let summary = Summary::new()
        .success("Installation complete".to_string())
        .warning("1 tool is outdated".to_string())
        .info("Run 'vex upgrade' to update".to_string());

    summary.render();
}

#[test]
fn test_progress_non_interactive() {
    let ctx = UiContext::non_interactive();
    let progress = Progress::new(&ctx, "Testing");
    progress.set_message("Still testing");
    progress.finish_with_success("Test complete");
}
