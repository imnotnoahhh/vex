use crate::templates::{MergeStrategy, TemplateFile};

use super::template_file;

pub(super) fn build(project_name: &str) -> Vec<TemplateFile> {
    vec![
        template_file(
            ".tool-versions",
            "go 1.24\n",
            Some(MergeStrategy::ToolVersions),
        ),
        template_file(
            ".vex.toml",
            r#"[commands]
fmt = "go fmt ./..."
build = "go build ./..."
test = "go test ./..."
run = "go run ./cmd/app"
"#,
            None,
        ),
        template_file(".gitignore", "bin/\n", Some(MergeStrategy::GitIgnore)),
        template_file(
            "go.mod",
            format!("module example.com/{}\n\ngo 1.24\n", project_name),
            None,
        ),
        template_file(
            "cmd/app/main.go",
            r#"package main

import "fmt"

func main() {
    fmt.Println("hello from vex")
}
"#,
            None,
        ),
        template_file(
            "tests/README.md",
            "Use `go test ./...` for package tests and add higher-level fixtures here if needed.\n",
            None,
        ),
        template_file(
            "cmd/app/main_test.go",
            r#"package main

import "testing"

func TestMainPackageBuilds(t *testing.T) {
    t.Log("Add service-level tests here.")
}
"#,
            None,
        ),
    ]
}
