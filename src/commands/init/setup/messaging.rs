use owo_colors::OwoColorize;
use std::path::Path;

pub(super) fn print_home_init_message(vex_dir: &Path, dry_run: bool) {
    println!(
        "{}  directory structure at {}",
        if dry_run {
            "Would create"
        } else {
            "✓ Created"
        }
        .bright_green(),
        vex_dir.display().to_string().dimmed()
    );
    println!();
}

pub(super) fn print_restart_instructions(shell_name: &str, config_path: &Path) {
    println!("{}", "Restart your shell or run:".dimmed());
    println!();
    match shell_name {
        "zsh" => println!("  source ~/.zshrc"),
        "bash" => {
            if config_path.ends_with(".bashrc") {
                println!("  source ~/.bashrc");
            } else {
                println!("  source ~/.bash_profile");
            }
        }
        "fish" => println!("  source ~/.config/fish/config.fish"),
        "nu" | "nushell" => println!("  source ~/.config/nushell/config.nu"),
        _ => {}
    }
    println!();
}

pub(super) fn print_skip_instructions() {
    println!("{}", "To enable auto-switching on cd, run:".dimmed());
    println!();
    println!("  vex init --shell auto");
    println!();
    println!("Or manually configure your shell:");
    println!();
    println!("  echo 'eval \"$(vex env zsh)\"' >> ~/.zshrc && source ~/.zshrc");
    println!();
}

pub(super) fn print_manual_shell_instructions() {
    println!(
        "{} Unable to detect shell. Please configure manually:",
        "⚠".yellow()
    );
    println!();
    println!("  vex init --shell zsh    # or bash, fish, nu");
    println!();
}
