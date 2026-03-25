use crate::advisories;
use crate::error::Result;
use owo_colors::OwoColorize;

pub(super) type InstallResult = (String, String, Result<bool>);

pub(super) fn print_install_summary(results: &[InstallResult]) {
    println!();
    println!("{}", "Sync Summary:".cyan().bold());

    let mut installed = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for (tool, version, result) in results {
        match result {
            Ok(true) => {
                println!("  {} {}@{}", "✓".green(), tool.yellow(), version.cyan());
                installed += 1;

                let advisory = advisories::get_advisory(tool, version);
                if advisory.is_warning() {
                    if let Some(message) = &advisory.message {
                        println!("    {} {}", "⚠".yellow(), message.dimmed());
                    }
                    if let Some(recommendation) = &advisory.recommendation {
                        println!("    {} {}", "→".cyan(), recommendation.dimmed());
                    }
                }
            }
            Ok(false) => {
                println!(
                    "  {} {}@{} (already installed)",
                    "→".dimmed(),
                    tool.yellow(),
                    version.cyan()
                );
                skipped += 1;
            }
            Err(error) => {
                println!(
                    "  {} {}@{}: {}",
                    "✗".red(),
                    tool.yellow(),
                    version.cyan(),
                    error
                );
                failed += 1;
            }
        }
    }

    println!();
    println!(
        "Installed: {}, Skipped: {}, Failed: {}",
        installed.to_string().green(),
        skipped.to_string().yellow(),
        failed.to_string().red()
    );
}
