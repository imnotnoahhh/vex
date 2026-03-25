mod catalog;
mod write;

use crate::error::Result;
use catalog::{render_template_plan, TEMPLATE_INFOS};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};
use write::plan::build_write_plan;
use write::rollback::apply_write_plan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictMode {
    Strict,
    AddOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct TemplateInfo {
    pub id: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum MergeStrategy {
    ToolVersions,
    GitIgnore,
}

#[derive(Debug, Clone)]
pub(super) struct TemplateFile {
    pub(super) path: &'static str,
    pub(super) contents: String,
    pub(super) merge_strategy: Option<MergeStrategy>,
}

#[derive(Debug, Clone)]
pub(super) enum PlannedWriteKind {
    Create,
    Merge,
}

#[derive(Debug, Clone)]
pub(super) struct PlannedWrite {
    pub(super) path: PathBuf,
    pub(super) contents: String,
    pub(super) kind: PlannedWriteKind,
}

pub fn list_templates() -> &'static [TemplateInfo] {
    TEMPLATE_INFOS
}

pub fn print_templates() {
    println!("{}", "Available templates:".cyan().bold());
    for template in list_templates() {
        println!("  {} {}", template.id.cyan(), template.description);
    }
}

pub fn init_template(
    cwd: &Path,
    template_name: &str,
    dry_run: bool,
    conflict_mode: ConflictMode,
) -> Result<()> {
    let plan = render_template_plan(cwd, template_name)?;
    let preview = build_write_plan(cwd, &plan, conflict_mode)?;

    if preview.is_empty() {
        println!(
            "{} Template {} is already satisfied in {}",
            "✓".green(),
            template_name.cyan(),
            cwd.display().to_string().dimmed()
        );
        return Ok(());
    }

    println!(
        "{} template {} in {}",
        if dry_run {
            "Previewing".bright_yellow().to_string()
        } else {
            "Initializing".bright_green().to_string()
        },
        template_name.cyan(),
        cwd.display().to_string().dimmed()
    );

    for item in &preview {
        match item.kind {
            PlannedWriteKind::Create => {
                println!("  {} {}", "create".green(), item.path.display());
            }
            PlannedWriteKind::Merge => {
                println!("  {} {}", "merge".yellow(), item.path.display());
            }
        }
    }

    if dry_run {
        println!();
        println!("{}", "No files were written (--dry-run).".dimmed());
        return Ok(());
    }

    apply_write_plan(cwd, &preview)?;

    println!();
    println!(
        "{} Created {} template files",
        "✓".green(),
        preview.len().to_string().cyan()
    );
    Ok(())
}

#[cfg(test)]
mod tests;
