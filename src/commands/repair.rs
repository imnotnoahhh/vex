use crate::cli::repair::{MigrateHomeArgs, RepairCommands};
use crate::error::{Result, VexError};
use crate::home_state::{self, AuditKind};
use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;

pub fn run(args: &crate::cli::repair::RepairArgs) -> Result<()> {
    match &args.command {
        RepairCommands::MigrateHome(migrate) => migrate_home(migrate),
    }
}

pub fn migrate_home(args: &MigrateHomeArgs) -> Result<()> {
    let home = dirs::home_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    let tool = args.tool.as_deref().unwrap_or("all");
    let audits = home_state::audit(&home, Some(tool));

    if audits.is_empty() {
        println!("{}", "No supported legacy home state was found.".green());
        return Ok(());
    }

    println!(
        "{} {}",
        if args.apply { "Applying" } else { "Previewing" }.cyan(),
        "home-state migration plan".cyan()
    );
    println!();

    let mut migrated = 0usize;
    let mut skipped = 0usize;

    for audit in audits {
        match audit.kind {
            AuditKind::Advisory => {
                println!("{} {}", "manual".yellow(), audit.summary);
                println!("  {}", audit.source.display());
                skipped += 1;
            }
            AuditKind::SafeMigration => {
                let Some(destination) = audit.destination.as_ref() else {
                    continue;
                };

                if audit.destination_exists {
                    println!(
                        "{} {} (destination already exists: {})",
                        "skip".yellow(),
                        audit.summary,
                        destination.display()
                    );
                    skipped += 1;
                    continue;
                }

                println!(
                    "{} {}",
                    if args.apply {
                        "move".green().to_string()
                    } else {
                        "would move".cyan().to_string()
                    },
                    audit.summary
                );
                println!("  {} -> {}", audit.source.display(), destination.display());

                if args.apply {
                    move_path(&audit.source, destination)?;
                    migrated += 1;
                }
            }
        }
    }

    println!();
    if args.apply {
        println!(
            "{} {} migrated, {} skipped",
            "Done.".green(),
            migrated,
            skipped
        );
    } else {
        println!(
            "{} run {} to apply these safe migrations.",
            "Dry run complete.".dimmed(),
            "vex repair migrate-home --apply".cyan()
        );
    }

    Ok(())
}

fn move_path(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(_) if source.is_file() => {
            fs::copy(source, destination)?;
            fs::remove_file(source)?;
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}
