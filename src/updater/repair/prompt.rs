use super::scan::BrokenVersion;
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::process::Command;

pub(super) fn report_scan_start() {
    println!(
        "\n{} Checking for broken installations from old vex...",
        "→".cyan()
    );
}

pub(super) fn report_no_broken_installations() {
    println!("{} No broken installations found", "✓".green());
}

pub(super) fn report_broken_installations(broken_versions: &[BrokenVersion]) {
    println!(
        "\n{} Found {} broken installation(s):",
        "!".yellow(),
        broken_versions.len()
    );
    for (tool, version) in broken_versions {
        println!("  • {}@{}", tool, version);
    }

    println!(
        "\n{}",
        "These versions were installed with an old vex version that had a symlink bug.".dimmed()
    );
    println!(
        "{}",
        "They need to be reinstalled to work correctly.".dimmed()
    );
}

pub(super) fn report_non_interactive_skip() {
    println!(
        "\n{} Non-interactive mode is enabled, so automatic repair is skipped.",
        "→".cyan()
    );
    println!(
        "{}",
        "Reinstall the affected versions manually once an interactive shell is available.".dimmed()
    );
}

pub(super) fn confirm_reinstall() -> io::Result<bool> {
    println!("\nReinstall them? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}

pub(super) fn report_skipped() {
    println!(
        "\n{} Skipped repair. Run 'vex doctor' to check again.",
        "→".cyan()
    );
}

pub(super) fn repair_versions(broken_versions: &[BrokenVersion]) {
    println!();
    for (tool, version) in broken_versions {
        println!("{} Reinstalling {}@{}...", "→".cyan(), tool, version);

        let status = Command::new("vex")
            .args([
                "install",
                &format!("{}@{}", tool, version),
                "--no-switch",
                "--force",
            ])
            .status();

        match status {
            Ok(exit) if exit.success() => {
                println!("{} Reinstalled {}@{}", "✓".green(), tool, version);
            }
            _ => {
                println!("{} Failed to reinstall {}@{}", "✗".red(), tool, version);
            }
        }
    }
}

pub(super) fn report_complete() {
    println!(
        "\n{} Repair complete. Run 'vex doctor' to verify.",
        "✓".green()
    );
}
