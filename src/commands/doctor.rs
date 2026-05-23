mod checks;
mod render;
mod types;

use crate::config;
use crate::error::{Result, VexError};
use crate::output::{print_json, OutputMode};
use owo_colors::OwoColorize;

pub use types::{CheckStatus, DoctorReport};

pub fn run_with_repair(output: OutputMode, verbose: bool, repair: bool) -> Result<()> {
    if repair {
        let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
        let removed = checks::repair_broken_links(&vex_dir);
        if matches!(output, OutputMode::Text) {
            if removed.is_empty() {
                println!("{} no dangling symlinks to repair", "✓".green());
            } else {
                println!(
                    "{} Removed {} dangling symlink{}:",
                    "✓".green(),
                    removed.len(),
                    if removed.len() == 1 { "" } else { "s" }
                );
                for label in &removed {
                    println!("  - {}", label);
                }
                println!();
            }
        }
    }

    let report = collect()?;
    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render::render_text(&report, verbose);
            Ok(())
        }
    }
}

pub fn collect() -> Result<DoctorReport> {
    checks::collect()
}
