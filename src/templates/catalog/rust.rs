use crate::templates::{MergeStrategy, TemplateFile};

use super::template_file;

pub(super) fn build(project_name: &str) -> Vec<TemplateFile> {
    vec![
        template_file(
            ".tool-versions",
            "rust stable\n",
            Some(MergeStrategy::ToolVersions),
        ),
        template_file(
            ".vex.toml",
            r#"[commands]
fmt = "cargo fmt"
build = "cargo build"
test = "cargo test"
run = "cargo run"
"#,
            None,
        ),
        template_file(".gitignore", "target/\n", Some(MergeStrategy::GitIgnore)),
        template_file(
            "Cargo.toml",
            format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
                project_name
            ),
            None,
        ),
        template_file(
            "src/main.rs",
            r#"fn main() {
    println!("hello from vex");
}
"#,
            None,
        ),
        template_file(
            "tests/smoke.rs",
            r#"#[test]
fn smoke_test() {
    assert_eq!(2 + 2, 4);
}
"#,
            None,
        ),
    ]
}
