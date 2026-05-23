mod home;
mod integration;
mod messaging;

use crate::error::{Result, VexError};
use crate::paths::vex_dir;
use crate::shell;
use home::initialize_vex_home;
use integration::{configure_shell_integration, remove_shell_integration_block, resolve_shell};
use messaging::{
    print_home_init_message, print_manual_shell_instructions, print_skip_instructions,
};
use owo_colors::OwoColorize;

pub(super) fn init_vex(shell_arg: &str, dry_run: bool) -> Result<()> {
    let vex_dir = vex_dir()?;
    initialize_vex_home(&vex_dir, dry_run)?;
    print_home_init_message(&vex_dir, dry_run);

    match resolve_shell(shell_arg)? {
        Some(shell_name) => configure_shell_integration(&shell_name, dry_run),
        None if shell_arg == "skip" => {
            print_skip_instructions();
            Ok(())
        }
        None => {
            print_manual_shell_instructions();
            Ok(())
        }
    }
}

pub(super) fn uninit_vex(shell_arg: &str, dry_run: bool) -> Result<()> {
    let Some(shell_name) = resolve_shell(shell_arg)? else {
        println!(
            "{} no shell selected — pass --shell zsh|bash|fish|nu to remove the hook",
            "ℹ".blue()
        );
        return Ok(());
    };

    let config_path = shell::get_shell_config_path(&shell_name).map_err(VexError::Parse)?;
    if !config_path.exists() {
        println!(
            "{} nothing to remove — {} does not exist",
            "ℹ".blue(),
            config_path.display().to_string().dimmed()
        );
        return Ok(());
    }

    if dry_run {
        println!(
            "{} Would strip vex hook block from {}",
            "Preview".bright_yellow(),
            config_path.display().to_string().dimmed()
        );
        return Ok(());
    }

    if remove_shell_integration_block(&config_path)? {
        println!(
            "{} Removed vex shell integration from {}",
            "✓".green(),
            config_path.display().to_string().dimmed()
        );
        println!(
            "  Restart your shell or run `exec {}` for the change to take effect.",
            shell_name
        );
    } else {
        println!(
            "{} no vex hook block found in {}",
            "ℹ".blue(),
            config_path.display().to_string().dimmed()
        );
    }
    Ok(())
}
