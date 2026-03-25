use super::{InstalledVersionsReport, RemoteVersionsReport};
use owo_colors::OwoColorize;

pub(super) fn render_installed_text(report: &InstalledVersionsReport) {
    if report.versions.is_empty() {
        println!("No versions of {} installed.", report.tool);
        return;
    }

    println!();
    println!("Installed versions of {}:", report.tool);
    println!();

    for version in &report.versions {
        if version.is_current {
            println!("  {} (current)", version.version);
        } else {
            println!("  {}", version.version);
        }
    }

    println!();
}

pub(super) fn render_remote_text(report: &RemoteVersionsReport) {
    if report.versions.is_empty() {
        println!("{}", "No versions found matching the filter.".yellow());
        return;
    }

    println!();
    println!("{} {} versions:", "Available".cyan(), report.tool.yellow());
    println!();

    let mut count = 0;
    for version in &report.versions {
        let mut visible = version.version.clone();
        if let Some(label) = &version.label {
            if report.tool == "python" {
                visible.push_str(&format!(" (Status: {})", label));
            } else {
                visible.push_str(&format!(" (LTS: {})", label));
            }
        }
        if version.is_current {
            visible.push_str(" ← current");
        }

        let mut display = if version.is_current {
            format!("{}", version.version.green().bold())
        } else if version.is_outdated {
            format!("{}", version.version.dimmed())
        } else {
            version.version.clone()
        };

        if let Some(label) = &version.label {
            let label = if report.tool == "python" {
                format!("(Status: {})", label)
            } else {
                format!("(LTS: {})", label)
            };
            display.push_str(&format!(" {}", label.cyan()));
        }
        if version.is_current {
            display.push_str(&format!(" {}", "← current".green()));
        }

        let col_width = 28;
        let padding = if visible.len() < col_width {
            " ".repeat(col_width - visible.len())
        } else {
            "  ".to_string()
        };

        print!("  {}{}", display, padding);
        count += 1;
        if count % 3 == 0 {
            println!();
        }
    }
    if count % 3 != 0 {
        println!();
    }

    println!();
    println!(
        "{} {} versions (filter: {})",
        "Total:".dimmed(),
        report.total,
        report.filter.dimmed()
    );
}
