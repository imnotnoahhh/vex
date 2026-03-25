use super::messaging::print_restart_instructions;
use crate::config;
use crate::error::{Result, VexError};
use crate::shell;
use owo_colors::OwoColorize;
use std::fs;

pub(super) fn resolve_shell(shell_arg: &str) -> Result<Option<String>> {
    match shell_arg {
        "auto" => Ok(config::default_shell()?.or_else(shell::detect_shell)),
        "skip" => Ok(None),
        _ => Ok(Some(shell_arg.to_string())),
    }
}

pub(super) fn configure_shell_integration(shell_name: &str, dry_run: bool) -> Result<()> {
    if let Err(error) = shell::generate_hook(shell_name) {
        eprintln!("{} {}", "✗".red(), error);
        return Ok(());
    }

    let config_path = shell::get_shell_config_path(shell_name).map_err(VexError::Parse)?;
    if shell::is_vex_configured(&config_path)? {
        println!(
            "{} vex is already configured in {}",
            "ℹ".blue(),
            config_path.display().to_string().dimmed()
        );
        println!();
        return Ok(());
    }

    let hook_command = shell_hook_command(shell_name)?;
    if dry_run {
        println!(
            "{} Would append to {}:",
            "Preview".bright_yellow(),
            config_path.display().to_string().dimmed()
        );
        println!("{}", hook_command.dimmed());
        return Ok(());
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)?;
    file.write_all(hook_command.as_bytes())?;

    println!(
        "{} Configured {} shell integration in {}",
        "✓".green(),
        shell_name.bright_cyan(),
        config_path.display().to_string().dimmed()
    );
    println!();
    print_restart_instructions(shell_name, &config_path);
    Ok(())
}

fn shell_hook_command(shell_name: &str) -> Result<String> {
    match shell_name {
        "zsh" => Ok("\n# vex shell integration\neval \"$(vex env zsh)\"\n".to_string()),
        "bash" => Ok("\n# vex shell integration\neval \"$(vex env bash)\"\n".to_string()),
        "fish" => Ok("\n# vex shell integration\nvex env fish | source\n".to_string()),
        "nu" | "nushell" => Ok(
            "\n# vex shell integration\nvex env nu | save -f ~/.vex-env.nu\nsource ~/.vex-env.nu\n"
                .to_string(),
        ),
        _ => Err(VexError::Parse(format!(
            "Unsupported shell: {}",
            shell_name
        ))),
    }
}
