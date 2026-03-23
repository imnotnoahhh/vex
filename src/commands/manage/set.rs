use crate::error::{Result, VexError};
use crate::paths::vex_dir;
use crate::resolver;
use crate::spec::parse_spec;
use crate::tools;
use crate::version_files;
use owo_colors::OwoColorize;
use std::path::Path;

pub fn set_project_version(spec: &str) -> Result<()> {
    let (tool_name, version) = parse_spec(spec)?;
    if version.is_empty() {
        return Err(VexError::Parse(
            "Please specify a version (e.g., node@20.11.0)".to_string(),
        ));
    }

    set_version(
        &resolver::current_dir().join(".tool-versions"),
        &tool_name,
        &version,
        true,
    )
}

pub fn set_global_version(spec: &str) -> Result<()> {
    let (tool_name, version) = parse_spec(spec)?;
    if version.is_empty() {
        return Err(VexError::Parse(
            "Please specify a version (e.g., node@20.11.0)".to_string(),
        ));
    }

    set_version(
        &vex_dir()?.join("tool-versions"),
        &tool_name,
        &version,
        false,
    )
}

fn set_version(file_path: &Path, tool_name: &str, version: &str, is_project: bool) -> Result<()> {
    let tool = tools::get_tool(tool_name)?;
    let resolved = tools::resolve_fuzzy_version(tool.as_ref(), version)?;
    let is_installed = vex_dir()?
        .join("toolchains")
        .join(tool_name)
        .join(&resolved)
        .exists();

    version_files::write_tool_version(file_path, tool_name, &resolved)?;

    println!(
        "{} {}: {}@{}",
        "✓".green(),
        if is_project {
            "Set project version"
        } else {
            "Set global default"
        },
        tool_name.yellow(),
        resolved.cyan()
    );

    print_install_hint(tool_name, &resolved, is_installed);
    println!("{}", format!("  Config: {}", file_path.display()).dimmed());
    print_activation_hint(is_project, is_installed);

    Ok(())
}

fn print_install_hint(tool_name: &str, resolved: &str, is_installed: bool) {
    if is_installed {
        return;
    }

    println!(
        "{}",
        format!(
            "  Note: Version {}@{} is not installed yet.",
            tool_name, resolved
        )
        .yellow()
    );
    println!(
        "{}",
        format!(
            "  Run 'vex install {}@{}' to install it.",
            tool_name, resolved
        )
        .dimmed()
    );
}

fn print_activation_hint(is_project: bool, is_installed: bool) {
    println!();
    if !is_project {
        println!(
            "{}",
            "This version will be used when no project-specific version is found.".dimmed()
        );
    }
    if is_installed {
        println!("{}", "To activate it now, run: vex use --auto".dimmed());
    }
}
