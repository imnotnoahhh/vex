use super::{MergeStrategy, TemplateFile, TemplateInfo};
use crate::error::{Result, VexError};
use std::path::Path;

mod go;
mod java;
mod node;
mod python;
mod rust;

pub(super) const TEMPLATE_INFOS: &[TemplateInfo] = &[
    TemplateInfo {
        id: "node-typescript",
        description: "TypeScript app with npm scripts and vex-managed Node.js",
    },
    TemplateInfo {
        id: "go-service",
        description: "Go service starter with cmd/app layout and vex-managed Go",
    },
    TemplateInfo {
        id: "java-basic",
        description: "Basic Java CLI starter aligned with vex-managed JDK workflows",
    },
    TemplateInfo {
        id: "rust-cli",
        description: "Rust CLI starter with Cargo and vex-managed stable Rust",
    },
    TemplateInfo {
        id: "python-venv",
        description: "Python starter aligned with vex python init/freeze/sync and .venv",
    },
];

pub(super) fn render_template_plan(cwd: &Path, template_name: &str) -> Result<Vec<TemplateFile>> {
    let project_name = inferred_project_name(cwd);

    match template_name {
        "node-typescript" => Ok(node::build(&project_name)),
        "go-service" => Ok(go::build(&project_name)),
        "java-basic" => Ok(java::build()),
        "rust-cli" => Ok(rust::build(&project_name)),
        "python-venv" => Ok(python::build()),
        _ => Err(VexError::Config(format!(
            "Unknown template '{}'. Run 'vex init --list-templates' to see the supported template names.",
            template_name
        ))),
    }
}

fn inferred_project_name(cwd: &Path) -> String {
    let fallback = "vex-app".to_string();
    let Some(name) = cwd.file_name().and_then(|value| value.to_str()) else {
        return fallback;
    };

    let mut sanitized = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == '_' || ch == '.' {
            sanitized.push('-');
        }
    }
    let sanitized = sanitized.trim_matches('-').to_string();
    if sanitized.is_empty() {
        fallback
    } else {
        sanitized
    }
}

pub(super) fn template_file(
    path: &'static str,
    contents: impl Into<String>,
    merge_strategy: Option<MergeStrategy>,
) -> TemplateFile {
    TemplateFile {
        path,
        contents: contents.into(),
        merge_strategy,
    }
}
