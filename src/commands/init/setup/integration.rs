use super::messaging::print_restart_instructions;
use crate::config;
use crate::error::{Result, VexError};
use crate::shell;
use owo_colors::OwoColorize;
use std::fs;

/// Marker lines wrapped around the vex hook so we can find and remove it cleanly.
pub(crate) const HOOK_BEGIN_MARKER: &str = "# >>> vex initialize >>>";
pub(crate) const HOOK_END_MARKER: &str = "# <<< vex initialize <<<";

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
    let body = match shell_name {
        "zsh" => "eval \"$(vex env zsh)\"".to_string(),
        "bash" => "eval \"$(vex env bash)\"".to_string(),
        "fish" => "vex env fish | source".to_string(),
        "nu" | "nushell" => "vex env nu | save -f ~/.vex-env.nu\nsource ~/.vex-env.nu".to_string(),
        _ => {
            return Err(VexError::Parse(format!(
                "Unsupported shell: {}",
                shell_name
            )));
        }
    };

    Ok(format!(
        "\n{begin}\n# vex shell integration (managed by `vex init` / `vex uninit`)\n{body}\n{end}\n",
        begin = HOOK_BEGIN_MARKER,
        body = body,
        end = HOOK_END_MARKER,
    ))
}

/// Remove the vex-managed hook block from a shell config file.
///
/// Returns `true` when content was removed, `false` when no marker block was found.
/// Falls back to stripping legacy unmarked hook lines for backwards compatibility.
pub(crate) fn remove_shell_integration_block(path: &std::path::Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let original = fs::read_to_string(path)?;
    let (stripped, removed) = strip_hook_block(&original);
    if !removed {
        return Ok(false);
    }
    fs::write(path, stripped)?;
    Ok(true)
}

fn strip_hook_block(content: &str) -> (String, bool) {
    let mut out = String::with_capacity(content.len());
    let mut in_block = false;
    let mut removed = false;
    let mut legacy_skip_body = false;

    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if !in_block && trimmed.trim() == HOOK_BEGIN_MARKER {
            in_block = true;
            removed = true;
            continue;
        }
        if in_block {
            if trimmed.trim() == HOOK_END_MARKER {
                in_block = false;
            }
            continue;
        }

        // Legacy: pre-marker installs wrote an unmarked block:
        //   # vex shell integration
        //   eval "$(vex env zsh)"   (or fish/nu/nu equivalent)
        if legacy_skip_body {
            if is_legacy_hook_body(trimmed.trim()) {
                removed = true;
                continue;
            }
            legacy_skip_body = false;
        }
        if trimmed.trim() == "# vex shell integration" {
            legacy_skip_body = true;
            removed = true;
            continue;
        }

        out.push_str(line);
    }

    // Collapse trailing blank lines introduced by the removal.
    let trimmed_out = out.trim_end_matches('\n').to_string() + "\n";
    (if removed { trimmed_out } else { out }, removed)
}

fn is_legacy_hook_body(line: &str) -> bool {
    matches!(
        line,
        "eval \"$(vex env zsh)\""
            | "eval \"$(vex env bash)\""
            | "vex env fish | source"
            | "vex env nu | save -f ~/.vex-env.nu"
            | "source ~/.vex-env.nu"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_marker_block() {
        let input = "alpha\n# >>> vex initialize >>>\n# vex shell integration\neval \"$(vex env zsh)\"\n# <<< vex initialize <<<\nbeta\n";
        let (out, removed) = strip_hook_block(input);
        assert!(removed);
        assert_eq!(out, "alpha\nbeta\n");
    }

    #[test]
    fn strips_legacy_two_line_block() {
        let input = "alpha\n# vex shell integration\neval \"$(vex env zsh)\"\nbeta\n";
        let (out, removed) = strip_hook_block(input);
        assert!(removed);
        assert_eq!(out, "alpha\nbeta\n");
    }

    #[test]
    fn strips_legacy_nushell_block() {
        let input = "alpha\n# vex shell integration\nvex env nu | save -f ~/.vex-env.nu\nsource ~/.vex-env.nu\nbeta\n";
        let (out, removed) = strip_hook_block(input);
        assert!(removed);
        assert_eq!(out, "alpha\nbeta\n");
    }

    #[test]
    fn untouched_when_no_block() {
        let input = "alpha\nbeta\n";
        let (out, removed) = strip_hook_block(input);
        assert!(!removed);
        assert_eq!(out, input);
    }
}
