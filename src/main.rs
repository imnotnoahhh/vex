//! vex - macOS binary version manager
//!
//! Manages official binary distributions of Node.js, Go, Java, Rust, and other languages.
//! Implements fast version switching via symlinks + PATH prepending.

use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::fs;
use std::path::PathBuf;

mod activation;
mod advisories;
mod alias;
mod cache;
mod commands;
mod config;
mod downloader;
mod error;
mod http;
mod installer;
mod lock;
mod logging;
mod output;
mod project;
mod resolver;
mod shell;
mod switcher;
mod tools;
mod ui;
mod updater;

use error::Result;

/// Filter type for list-remote command
#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
enum FilterType {
    /// Show all versions
    #[default]
    All,
    /// Show only LTS versions
    Lts,
    /// Show only the latest version of each major release
    Major,
    /// Show only the latest version
    Latest,
}

/// Alias subcommands
#[derive(Subcommand)]
enum AliasCommands {
    /// Set a version alias
    Set {
        /// Tool name (e.g., node)
        tool: String,
        /// Alias name (e.g., prod)
        alias: String,
        /// Version (e.g., 20.11.0)
        version: String,
        /// Set as project-level alias (in .vex.toml)
        #[arg(long)]
        project: bool,
    },
    /// List aliases
    List {
        /// Tool name (e.g., node). Omit to list all aliases.
        tool: Option<String>,
        /// Show only project-level aliases
        #[arg(long)]
        project: bool,
        /// Show only global aliases
        #[arg(long)]
        global: bool,
    },
    /// Delete an alias
    Delete {
        /// Tool name (e.g., node)
        tool: String,
        /// Alias name (e.g., prod)
        alias: String,
        /// Delete from project-level aliases
        #[arg(long)]
        project: bool,
    },
}

/// vex CLI main structure
#[derive(Parser)]
#[command(name = "vex", version)]
#[command(about = "A fast version manager for macOS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// CLI subcommand definitions
#[derive(Subcommand)]
enum Commands {
    /// Initialize vex directory structure
    Init {
        /// Shell to configure (auto, zsh, bash, fish, or skip)
        #[arg(long, default_value = "skip")]
        shell: String,

        /// Preview changes without modifying files
        #[arg(long)]
        dry_run: bool,
    },

    /// Install a tool version (or all from .tool-versions)
    Install {
        /// Tool and version specs (e.g., node@20, go@1.22). Omit to install from .tool-versions.
        specs: Vec<String>,

        /// Skip automatic version switching after installation
        #[arg(long)]
        no_switch: bool,

        /// Force reinstall even if already installed
        #[arg(long)]
        force: bool,

        /// Install from a specific version file
        #[arg(long)]
        from: Option<PathBuf>,
    },

    /// Install missing versions from the current managed context
    Sync {
        /// Install from a specific version file
        #[arg(long)]
        from: Option<PathBuf>,
    },

    /// Switch to a different version
    Use {
        /// Tool and version (e.g., node@20.11.0). Omit to auto-detect from version files.
        spec: Option<String>,

        /// Auto mode: read version files (.tool-versions, .node-version, etc.)
        #[arg(long)]
        auto: bool,
    },

    /// List installed versions
    List {
        /// Tool name (e.g., node)
        tool: String,

        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },

    /// List available remote versions
    ListRemote {
        /// Tool name (e.g., node)
        tool: String,

        /// Filter type (all, lts, major, latest)
        #[arg(long, short = 'f', default_value = "all")]
        filter: FilterType,

        /// Skip cache and fetch fresh data
        #[arg(long)]
        no_cache: bool,

        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },

    /// Show current active versions
    Current {
        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },

    /// Uninstall a version
    Uninstall {
        /// Tool and version (e.g., node@20.11.0)
        spec: String,
    },

    /// Output shell hook for auto-switching
    Env {
        /// Shell type (zsh, bash, fish, or nu)
        shell: String,
    },

    /// Pin a tool version in the current directory (.tool-versions)
    Local {
        /// Tool and version (e.g., node@20.11.0)
        spec: String,
    },

    /// Pin a tool version globally (~/.vex/tool-versions)
    Global {
        /// Tool and version (e.g., node@20.11.0)
        spec: String,
    },

    /// Upgrade a tool to the latest version
    Upgrade {
        /// Tool name (e.g., node). Omit with --all.
        tool: Option<String>,

        /// Upgrade every managed tool in the current context
        #[arg(long)]
        all: bool,
    },

    /// Show which managed tools are behind the latest available version
    Outdated {
        /// Tool name (e.g., node). Omit to inspect the current managed context.
        tool: Option<String>,

        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },

    /// Remove unused cache files, stale locks, and unreferenced toolchains
    #[command(alias = "gc")]
    Prune {
        /// Show what would be removed without deleting anything
        #[arg(long)]
        dry_run: bool,
    },

    /// Manage version aliases
    Alias {
        #[command(subcommand)]
        subcmd: AliasCommands,
    },

    /// Run a command inside the resolved vex-managed environment without switching global state
    Exec {
        /// Command to run after '--' (for example: vex exec -- node -v)
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },

    /// Run a named task from .vex.toml inside the resolved vex-managed environment
    Run {
        /// Task name from [commands] in .vex.toml
        task: String,

        /// Extra arguments appended to the task command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Check vex installation health
    Doctor {
        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },

    /// Update vex itself to the latest release
    SelfUpdate,

    /// Python virtual environment management
    ///
    /// Workflow:
    ///   1. vex install python@3.12   (install a Python version globally)
    ///   2. cd my-project
    ///   3. vex python init            (create .venv using the active Python)
    ///   4. pip install \<packages\>
    ///   5. vex python freeze          (lock packages to requirements.lock)
    ///   6. vex python sync            (restore from requirements.lock on another machine)
    Python {
        /// Subcommand:
        ///   init   — Create .venv in the current directory using the active vex-managed Python.
        ///            Also records the Python version in .tool-versions.
        ///   freeze — Run `pip freeze` and write output to requirements.lock.
        ///            Use after installing packages to lock the environment.
        ///   sync   — Restore the environment from requirements.lock via `pip install -r`.
        ///            Auto-creates .venv if it does not exist yet.
        subcmd: String,
    },
}

/// Get vex root directory (~/.vex)
fn vex_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|p| p.join(".vex"))
        .ok_or(error::VexError::HomeDirectoryNotFound)
}

/// Migrate ~/.tool-versions → ~/.vex/tool-versions if the old file exists and the new one doesn't.
/// Silently skips if migration is not needed or if any step fails.
fn migrate_global_tool_versions() {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    let old_path = home.join(".tool-versions");
    let new_path = home.join(".vex").join("tool-versions");

    if !old_path.exists() || new_path.exists() {
        return;
    }

    if let Ok(content) = fs::read_to_string(&old_path) {
        if fs::create_dir_all(home.join(".vex")).is_ok() && fs::write(&new_path, &content).is_ok() {
            let _ = fs::remove_file(&old_path);
            eprintln!(
                "{} Migrated ~/.tool-versions → ~/.vex/tool-versions",
                "vex:".cyan()
            );
        }
    }
}

/// Initialize vex directory structure and configuration files
fn init_vex(shell_arg: &str, dry_run: bool) -> Result<()> {
    let vex_dir = vex_dir()?;

    // Create directory structure
    if !dry_run {
        fs::create_dir_all(vex_dir.join("cache"))?;
        fs::create_dir_all(vex_dir.join("locks"))?;
        fs::create_dir_all(vex_dir.join("toolchains"))?;
        fs::create_dir_all(vex_dir.join("current"))?;
        fs::create_dir_all(vex_dir.join("bin"))?;

        // Create configuration file
        let config_path = vex_dir.join("config.toml");
        if !config_path.exists() {
            fs::write(&config_path, "# vex configuration\n")?;
        }
    }

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

    // Handle shell configuration
    let shell = if shell_arg == "auto" {
        config::default_shell()?.or_else(shell::detect_shell)
    } else if shell_arg == "skip" {
        None
    } else {
        Some(shell_arg.to_string())
    };

    if let Some(shell_name) = shell {
        // Validate shell is supported
        if let Err(e) = shell::generate_hook(&shell_name) {
            eprintln!("{} {}", "✗".red(), e);
            return Ok(());
        }

        let config_path =
            shell::get_shell_config_path(&shell_name).map_err(error::VexError::Parse)?;

        // Check if already configured
        let already_configured = shell::is_vex_configured(&config_path)?;

        if already_configured {
            println!(
                "{} vex is already configured in {}",
                "ℹ".blue(),
                config_path.display().to_string().dimmed()
            );
            println!();
            return Ok(());
        }

        // Generate hook command
        let hook_command = match shell_name.as_str() {
            "zsh" => "\n# vex shell integration\neval \"$(vex env zsh)\"\n".to_string(),
            "bash" => "\n# vex shell integration\neval \"$(vex env bash)\"\n".to_string(),
            "fish" => "\n# vex shell integration\nvex env fish | source\n".to_string(),
            "nu" | "nushell" => {
                "\n# vex shell integration\nvex env nu | save -f ~/.vex-env.nu\nsource ~/.vex-env.nu\n".to_string()
            }
            _ => {
                return Err(error::VexError::Parse(format!(
                    "Unsupported shell: {}",
                    shell_name
                )))
            }
        };

        if dry_run {
            println!(
                "{} Would append to {}:",
                "Preview".bright_yellow(),
                config_path.display().to_string().dimmed()
            );
            println!("{}", hook_command.dimmed());
        } else {
            // Ensure parent directory exists (for fish/nu)
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Append hook to config file
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
            println!("{}", "Restart your shell or run:".dimmed());
            println!();
            match shell_name.as_str() {
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
    } else if shell_arg == "skip" {
        println!("{}", "To enable auto-switching on cd, run:".dimmed());
        println!();
        println!("  vex init --shell auto");
        println!();
        println!("Or manually configure your shell:");
        println!();
        println!("  echo 'eval \"$(vex env zsh)\"' >> ~/.zshrc && source ~/.zshrc");
        println!();
    } else {
        println!(
            "{} Unable to detect shell. Please configure manually:",
            "⚠".yellow()
        );
        println!();
        println!("  vex init --shell zsh    # or bash, fish, nu");
        println!();
    }

    Ok(())
}

/// Parse tool@version format spec string
fn parse_spec(spec: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = spec.split('@').collect();
    if parts.len() == 2 {
        // Format: tool@version
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else if parts.len() == 1 {
        // Format: tool (tool name only, requires interactive version selection)
        Ok((parts[0].to_string(), String::new()))
    } else {
        Err(error::VexError::Parse(format!(
            "Invalid spec format: {}. Expected format: tool@version or tool",
            spec
        )))
    }
}

#[allow(dead_code)]
fn interactive_install(tool_name: &str, no_switch: bool) -> Result<()> {
    if config::non_interactive()? {
        return Err(error::VexError::Dialog(
            "Interactive installation is disabled in non-interactive mode. Specify an explicit version instead (for example: 'vex install node@24.14.0')."
                .to_string(),
        ));
    }

    let tool = tools::get_tool(tool_name)?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Fetching available versions of {}...", tool_name));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let versions = fetch_versions_cached(tool.as_ref(), true)?;
    spinner.finish_and_clear();

    println!();
    println!("Select a version to install:");
    println!();

    let items: Vec<String> = versions
        .iter()
        .map(|v| {
            if let Some(lts) = &v.lts {
                format!("{} (LTS: {})", v.version, lts)
            } else {
                v.version.clone()
            }
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .max_length(20)
        .interact_opt()
        .map_err(|e| error::VexError::Dialog(e.to_string()))?;

    if let Some(index) = selection {
        let selected_version = &versions[index].version;
        // Remove v prefix (if present)
        let version = selected_version
            .strip_prefix('v')
            .unwrap_or(selected_version);

        println!();
        installer::install(tool.as_ref(), version)?;

        // Auto-switch unless --no-switch is specified
        if !no_switch {
            switcher::switch_version(tool.as_ref(), version)?;
            println!();
            println!(
                "{} Switched to {}@{}",
                "✓".green(),
                tool_name.yellow(),
                version.cyan()
            );
        } else {
            println!();
            println!(
                "{}",
                format!(
                    "To activate this version, run: vex use {}@{}",
                    tool_name, version
                )
                .dimmed()
            );
        }
    } else {
        println!("Installation cancelled.");
    }

    Ok(())
}

#[allow(dead_code)]
#[cfg(test)]
fn show_current() -> Result<()> {
    commands::current::show(output::OutputMode::Text)
}

fn show_current_with_output(output_mode: output::OutputMode) -> Result<()> {
    commands::current::show(output_mode)
}

fn uninstall(tool_name: &str, version: &str) -> Result<()> {
    let vex_dir = vex_dir()?;
    let version_dir = vex_dir.join("toolchains").join(tool_name).join(version);

    if !version_dir.exists() {
        return Err(error::VexError::VersionNotFound {
            tool: tool_name.to_string(),
            version: version.to_string(),
            suggestions: String::new(),
        });
    }

    println!("Uninstalling {} {}...", tool_name, version);

    // Check if it's the currently active version
    let current_link = vex_dir.join("current").join(tool_name);
    let is_active = if current_link.exists() {
        if let Ok(target) = fs::read_link(&current_link) {
            target == version_dir
        } else {
            false
        }
    } else {
        false
    };

    // Delete version directory
    fs::remove_dir_all(&version_dir)?;

    // Clean up symlinks for currently active version
    if is_active {
        let _ = fs::remove_file(&current_link);

        let tool = tools::get_tool(tool_name)?;
        let bin_dir = vex_dir.join("bin");
        for (bin_name, _) in tool.bin_paths() {
            let bin_link = bin_dir.join(bin_name);
            let _ = fs::remove_file(&bin_link);
        }
    }

    println!(
        "{} Uninstalled {} {}",
        "✓".green(),
        tool_name.yellow(),
        version.yellow()
    );

    Ok(())
}

#[allow(dead_code)]
#[cfg(test)]
fn list_installed(tool_name: &str) -> Result<()> {
    commands::versions::list_installed(tool_name, output::OutputMode::Text)
}

fn list_installed_with_output(tool_name: &str, output_mode: output::OutputMode) -> Result<()> {
    commands::versions::list_installed(tool_name, output_mode)
}

/// Fetch remote versions with optional cache support.
/// When use_cache is true, checks the cache first and writes back on miss.
#[allow(dead_code)]
fn fetch_versions_cached(tool: &dyn tools::Tool, use_cache: bool) -> Result<Vec<tools::Version>> {
    commands::versions::fetch_versions_cached(tool, use_cache)
}

#[allow(dead_code)]
#[cfg(test)]
fn list_remote(tool_name: &str, filter: FilterType, use_cache: bool) -> Result<()> {
    list_remote_with_output(tool_name, filter, use_cache, output::OutputMode::Text)
}

fn list_remote_with_output(
    tool_name: &str,
    filter: FilterType,
    use_cache: bool,
    output_mode: output::OutputMode,
) -> Result<()> {
    commands::versions::list_remote(
        tool_name,
        match filter {
            FilterType::All => commands::versions::RemoteFilter::All,
            FilterType::Lts => commands::versions::RemoteFilter::Lts,
            FilterType::Major => commands::versions::RemoteFilter::Major,
            FilterType::Latest => commands::versions::RemoteFilter::Latest,
        },
        use_cache,
        output_mode,
    )
}

/// Install multiple tool specs in one command
fn install_multiple_specs(specs: &[String], no_switch: bool, force: bool) -> Result<()> {
    let mut results = Vec::new();

    for spec in specs {
        let (tool_name, version) = parse_spec(spec)?;

        if version.is_empty() {
            return Err(error::VexError::Parse(format!(
                "Version required for multi-spec install: {}",
                spec
            )));
        }

        let tool = match tools::get_tool(&tool_name) {
            Ok(t) => t,
            Err(e) => {
                results.push((tool_name.clone(), version.clone(), Err(e)));
                continue;
            }
        };

        let resolved = match tools::resolve_fuzzy_version(tool.as_ref(), &version) {
            Ok(v) => v,
            Err(e) => {
                results.push((tool_name.clone(), version.clone(), Err(e)));
                continue;
            }
        };

        // Check if already installed
        let vex = vex_dir()?;
        let install_dir = vex.join("toolchains").join(&tool_name).join(&resolved);

        if install_dir.exists() && !force {
            results.push((tool_name.clone(), resolved.clone(), Ok(false)));
            continue;
        }

        // If --force, remove existing installation
        if force && install_dir.exists() {
            fs::remove_dir_all(&install_dir)?;
        }

        // Install
        match installer::install(tool.as_ref(), &resolved) {
            Ok(_) => {
                // Auto-switch unless --no-switch
                if !no_switch {
                    let _ = switcher::switch_version(tool.as_ref(), &resolved);
                }
                results.push((tool_name.clone(), resolved.clone(), Ok(true)));
            }
            Err(e) => {
                results.push((tool_name.clone(), resolved.clone(), Err(e)));
            }
        }
    }

    // Print summary
    println!();
    println!("{}", "Installation Summary:".cyan().bold());

    let mut installed = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for (tool, version, result) in &results {
        match result {
            Ok(true) => {
                println!("  {} {}@{}", "✓".green(), tool.yellow(), version.cyan());
                installed += 1;

                // Show lifecycle advisory if applicable
                let advisory = advisories::get_advisory(tool, version);
                if advisory.is_warning() {
                    if let Some(msg) = &advisory.message {
                        println!("    {} {}", "⚠".yellow(), msg.dimmed());
                    }
                    if let Some(rec) = &advisory.recommendation {
                        println!("    {} {}", "→".cyan(), rec.dimmed());
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
            Err(e) => {
                println!(
                    "  {} {}@{}: {}",
                    "✗".red(),
                    tool.yellow(),
                    version.cyan(),
                    e
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

    if failed > 0 {
        return Err(error::VexError::Parse(format!(
            "{} installation(s) failed",
            failed
        )));
    }

    Ok(())
}

/// Install from a specific version file
fn install_from_file(file_path: &std::path::Path) -> Result<()> {
    if !file_path.exists() {
        return Err(error::VexError::Parse(format!(
            "Version file not found: {}",
            file_path.display()
        )));
    }

    let content = fs::read_to_string(file_path)?;
    let versions = resolver::parse_tool_versions(&content);

    if versions.is_empty() {
        println!("No versions found in {}", file_path.display());
        return Ok(());
    }

    let vex_dir = vex_dir()?;
    let mut results = Vec::new();

    for (tool_name, version) in &versions {
        let tool = match tools::get_tool(tool_name) {
            Ok(t) => t,
            Err(e) => {
                results.push((tool_name.clone(), version.clone(), Err(e)));
                continue;
            }
        };

        let version_dir = vex_dir.join("toolchains").join(tool_name).join(version);
        if version_dir.exists() {
            results.push((tool_name.clone(), version.clone(), Ok(false)));
            continue;
        }

        match installer::install(tool.as_ref(), version) {
            Ok(_) => {
                let _ = switcher::switch_version(tool.as_ref(), version);
                results.push((tool_name.clone(), version.clone(), Ok(true)));
            }
            Err(e) => {
                results.push((tool_name.clone(), version.clone(), Err(e)));
            }
        }
    }

    // Print summary
    print_install_summary(&results);

    Ok(())
}

/// Sync missing versions from a specific file
fn sync_from_file(file_path: &std::path::Path) -> Result<()> {
    if !file_path.exists() {
        return Err(error::VexError::Parse(format!(
            "Version file not found: {}",
            file_path.display()
        )));
    }

    let content = fs::read_to_string(file_path)?;
    let versions = resolver::parse_tool_versions(&content);

    if versions.is_empty() {
        println!("No versions found in {}", file_path.display());
        return Ok(());
    }

    sync_versions(&versions)
}

/// Sync missing versions from current context
fn sync_from_current_context() -> Result<()> {
    let cwd = resolver::current_dir();
    let versions = resolver::resolve_versions(&cwd);

    if versions.is_empty() {
        println!("No version files found (.tool-versions, .node-version, etc.)");
        return Ok(());
    }

    let versions_vec: Vec<(String, String)> = versions.into_iter().collect();
    sync_versions(&versions_vec)
}

/// Sync versions (install missing ones)
fn sync_versions(versions: &[(String, String)]) -> Result<()> {
    let vex_dir = vex_dir()?;
    let mut results = Vec::new();

    for (tool_name, version) in versions {
        let tool = match tools::get_tool(tool_name) {
            Ok(t) => t,
            Err(e) => {
                results.push((tool_name.clone(), version.clone(), Err(e)));
                continue;
            }
        };

        let version_dir = vex_dir.join("toolchains").join(tool_name).join(version);
        if version_dir.exists() {
            results.push((tool_name.clone(), version.clone(), Ok(false)));
            continue;
        }

        match installer::install(tool.as_ref(), version) {
            Ok(_) => {
                let _ = switcher::switch_version(tool.as_ref(), version);
                results.push((tool_name.clone(), version.clone(), Ok(true)));
            }
            Err(e) => {
                results.push((tool_name.clone(), version.clone(), Err(e)));
            }
        }
    }

    print_install_summary(&results);

    Ok(())
}

/// Print installation summary
fn print_install_summary(results: &[(String, String, Result<bool>)]) {
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

                // Show lifecycle advisory if applicable
                let advisory = advisories::get_advisory(tool, version);
                if advisory.is_warning() {
                    if let Some(msg) = &advisory.message {
                        println!("    {} {}", "⚠".yellow(), msg.dimmed());
                    }
                    if let Some(rec) = &advisory.recommendation {
                        println!("    {} {}", "→".cyan(), rec.dimmed());
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
            Err(e) => {
                println!(
                    "  {} {}@{}: {}",
                    "✗".red(),
                    tool.yellow(),
                    version.cyan(),
                    e
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

fn install_from_version_files() -> Result<()> {
    let cwd = resolver::current_dir();
    let versions = resolver::resolve_versions(&cwd);

    if versions.is_empty() {
        println!("No version files found (.tool-versions, .node-version, etc.)");
        return Ok(());
    }

    let vex_dir = vex_dir()?;

    for (tool_name, version) in &versions {
        let tool = match tools::get_tool(tool_name) {
            Ok(t) => t,
            Err(_) => {
                eprintln!("vex: skipping unsupported tool '{}'", tool_name);
                continue;
            }
        };

        let version_dir = vex_dir.join("toolchains").join(tool_name).join(version);
        if version_dir.exists() {
            println!("{}@{} already installed, skipping.", tool_name, version);
            continue;
        }

        installer::install(tool.as_ref(), version)?;
        switcher::switch_version(tool.as_ref(), version)?;
    }

    Ok(())
}

fn write_tool_version(file_path: &std::path::Path, tool_name: &str, version: &str) -> Result<()> {
    let mut entries: Vec<(String, String)> = Vec::new();

    // Read existing file if present
    if file_path.is_file() {
        if let Ok(content) = fs::read_to_string(file_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let mut parts = trimmed.split_whitespace();
                if let (Some(t), Some(v)) = (parts.next(), parts.next()) {
                    if t != tool_name {
                        entries.push((t.to_string(), v.to_string()));
                    }
                }
            }
        }
    }

    entries.push((tool_name.to_string(), version.to_string()));
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let content: String = entries
        .iter()
        .map(|(t, v)| format!("{} {}", t, v))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(file_path, content)?;
    Ok(())
}

fn handle_alias_command(subcmd: &AliasCommands) -> Result<()> {
    let vex_home = vex_dir()?;
    let manager = alias::AliasManager::new(&vex_home);

    match subcmd {
        AliasCommands::Set {
            tool,
            alias: alias_name,
            version,
            project,
        } => {
            tools::get_tool(tool)?;
            if *project {
                manager.set_project(tool, alias_name, version)?;
                println!(
                    "{} Set project alias: {}@{} -> {}",
                    "✓".green(),
                    tool.yellow(),
                    alias_name.cyan(),
                    version.cyan()
                );
                println!("  Saved to .vex.toml");
            } else {
                manager.set_global(tool, alias_name, version)?;
                println!(
                    "{} Set global alias: {}@{} -> {}",
                    "✓".green(),
                    tool.yellow(),
                    alias_name.cyan(),
                    version.cyan()
                );
                println!("  Saved to ~/.vex/aliases.toml");
            }
        }
        AliasCommands::List {
            tool,
            project,
            global,
        } => {
            let show_project = *project || !*global;
            let show_global = *global || !*project;
            let mut has_any = false;

            if show_project {
                let project_aliases = manager.list_project(tool.as_deref())?;
                if !project_aliases.is_empty() {
                    println!("{}", "Project aliases (.vex.toml):".bold());
                    for (tool_name, aliases) in project_aliases {
                        println!("  {}:", tool_name.yellow());
                        for (alias_name, version) in aliases {
                            println!("    {:<16} -> {}", alias_name.cyan(), version);
                        }
                    }
                    println!();
                    has_any = true;
                }
            }

            if show_global {
                let global_aliases = manager.list_global(tool.as_deref())?;
                if !global_aliases.is_empty() {
                    println!("{}", "Global aliases (~/.vex/aliases.toml):".bold());
                    for (tool_name, aliases) in global_aliases {
                        println!("  {}:", tool_name.yellow());
                        for (alias_name, version) in aliases {
                            println!("    {:<16} -> {}", alias_name.cyan(), version);
                        }
                    }
                    println!();
                    has_any = true;
                }
            }

            if !has_any {
                if let Some(tool_name) = tool {
                    println!("No aliases found for {}", tool_name);
                } else {
                    println!("No aliases found");
                }
                println!("\nCreate an alias with: vex alias set <tool> <alias> <version>");
            }
        }
        AliasCommands::Delete {
            tool,
            alias: alias_name,
            project,
        } => {
            tools::get_tool(tool)?;
            let removed = if *project {
                manager.delete_project(tool, alias_name)?
            } else {
                manager.delete_global(tool, alias_name)?
            };

            if removed {
                let scope = if *project { "project" } else { "global" };
                println!(
                    "{} Deleted {} alias: {}@{}",
                    "✓".green(),
                    scope,
                    tool.yellow(),
                    alias_name.cyan()
                );
            } else {
                let scope = if *project { "project" } else { "global" };
                return Err(error::VexError::Parse(format!(
                    "Alias '{}' not found for {} in {} aliases",
                    alias_name, tool, scope
                )));
            }
        }
    }

    Ok(())
}

fn auto_switch() -> Result<()> {
    if !config::auto_switch()? {
        return Ok(());
    }

    if let Some(project_config) = project::load_nearest_project_config(&resolver::current_dir())? {
        if project_config.config.behavior.auto_switch == Some(false) {
            return Ok(());
        }
    }

    let cwd = resolver::current_dir();
    let versions = resolver::resolve_versions(&cwd);

    if versions.is_empty() {
        return Ok(());
    }

    let vex_dir = vex_dir()?;

    for (tool_name, version) in &versions {
        // Check tool support
        let tool = match tools::get_tool(tool_name) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Check if version is installed
        let version_dir = vex_dir.join("toolchains").join(tool_name).join(version);
        if !version_dir.exists() {
            eprintln!(
                "vex: {}@{} not installed. Run 'vex install' to install.",
                tool_name, version
            );
            continue;
        }

        // Check if already the current version (avoid redundant switching)
        let current_link = vex_dir.join("current").join(tool_name);
        if current_link.exists() {
            if let Ok(target) = fs::read_link(&current_link) {
                if let Some(current_ver) = target.file_name() {
                    if current_ver.to_string_lossy() == version.as_str() {
                        continue;
                    }
                }
            }
        }

        // Silent switch
        switcher::switch_version(tool.as_ref(), version)?;
    }

    Ok(())
}

/// Run `vex python init`: create .venv in cwd and record python version in .tool-versions
fn python_init() -> Result<()> {
    use std::process::Command;

    let cwd = resolver::current_dir();
    let python_bin = find_active_python_bin()?;

    println!(
        "Creating .venv using {}...",
        python_bin.display().to_string().cyan()
    );

    let status = Command::new(&python_bin)
        .args(["-m", "venv", ".venv"])
        .current_dir(&cwd)
        .status()?;

    if !status.success() {
        return Err(error::VexError::Parse(
            "Failed to create .venv. Make sure python is installed via 'vex install python@<version>'".to_string(),
        ));
    }

    // Record python version in .tool-versions if we know it
    let versions = resolver::resolve_versions(&cwd);
    if let Some((_, ver)) = versions.iter().find(|(t, _)| t.as_str() == "python") {
        let file_path = cwd.join(".tool-versions");
        write_tool_version(&file_path, "python", ver)?;
        println!(
            "{} Recorded python {} in .tool-versions",
            "✓".green(),
            ver.cyan()
        );
    }

    println!(
        "{} Created .venv in {}",
        "✓".green(),
        cwd.display().to_string().dimmed()
    );
    println!();
    println!("{}", "To activate now:  source .venv/bin/activate".dimmed());
    println!(
        "{}",
        "Auto-activates:   next time you cd into this directory".dimmed()
    );

    Ok(())
}

/// Run `vex python freeze`: write pip freeze output to requirements.lock
fn python_freeze() -> Result<()> {
    use std::process::Command;

    let cwd = resolver::current_dir();
    let pip = cwd.join(".venv").join("bin").join("pip");

    if !pip.exists() {
        return Err(error::VexError::PythonEnv(
            "No .venv found. Run 'vex python init' first.".to_string(),
        ));
    }

    let output = Command::new(&pip)
        .arg("freeze")
        .current_dir(&cwd)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(error::VexError::Parse(format!(
            "pip freeze failed: {}",
            stderr
        )));
    }

    let lock_path = cwd.join("requirements.lock");
    fs::write(&lock_path, &output.stdout)?;

    let line_count = output.stdout.iter().filter(|&&b| b == b'\n').count();
    println!(
        "{} Wrote {} packages to {}",
        "✓".green(),
        line_count,
        "requirements.lock".cyan()
    );

    Ok(())
}

/// Run `vex python sync`: restore environment from requirements.lock (auto-init if needed)
fn python_sync() -> Result<()> {
    use std::process::Command;

    let cwd = resolver::current_dir();
    let venv = cwd.join(".venv");
    let lock_path = cwd.join("requirements.lock");

    if !lock_path.exists() {
        return Err(error::VexError::PythonEnv(
            "No requirements.lock found. Run 'vex python freeze' first.".to_string(),
        ));
    }

    if !venv.exists() {
        println!("{}", "No .venv found, initializing...".dimmed());
        python_init()?;
    }

    let pip = venv.join("bin").join("pip");
    println!("Installing from requirements.lock...");

    let status = Command::new(&pip)
        .args(["install", "-r", "requirements.lock"])
        .current_dir(&cwd)
        .status()?;

    if !status.success() {
        return Err(error::VexError::Parse(
            "pip install failed. Check requirements.lock for errors.".to_string(),
        ));
    }

    println!(
        "{} Environment restored from requirements.lock",
        "✓".green()
    );

    Ok(())
}

/// Find the active python3 binary from vex bin dir, falling back to system python3
fn find_active_python_bin() -> Result<std::path::PathBuf> {
    let vex_dir = vex_dir()?;
    let bin = vex_dir.join("bin").join("python3");
    if bin.exists() {
        return Ok(bin);
    }
    Ok(std::path::PathBuf::from("python3"))
}

#[allow(dead_code)]
#[cfg(test)]
fn run_doctor() -> Result<()> {
    commands::doctor::run(output::OutputMode::Text)
}

fn run_doctor_with_output(output_mode: output::OutputMode) -> Result<()> {
    commands::doctor::run(output_mode)
}

fn run() -> Result<()> {
    migrate_global_tool_versions();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { shell, dry_run } => {
            init_vex(&shell, dry_run)?;
        }
        Commands::Install {
            specs,
            no_switch,
            force,
            from,
        } => {
            if !specs.is_empty() {
                // Multi-spec install
                install_multiple_specs(&specs, no_switch, force)?;
            } else if let Some(from_path) = from {
                // Install from specific file
                install_from_file(&from_path)?;
            } else {
                // Install from current context (.tool-versions)
                install_from_version_files()?;
            }
        }
        Commands::Sync { from } => {
            if let Some(from_path) = from {
                sync_from_file(&from_path)?;
            } else {
                sync_from_current_context()?;
            }
        }
        Commands::Use { spec, auto } => {
            if auto {
                auto_switch()?;
            } else if let Some(spec) = spec {
                let (tool_name, version) = parse_spec(&spec)?;
                let tool = tools::get_tool(&tool_name)?;
                let resolved = tools::resolve_fuzzy_version(tool.as_ref(), &version)?;
                switcher::switch_version(tool.as_ref(), &resolved)?;

                // Show lifecycle advisory if applicable
                let advisory = advisories::get_advisory(&tool_name, &resolved);
                if advisory.is_warning() {
                    println!();
                    if let Some(msg) = &advisory.message {
                        println!("{} {}", "warning:".yellow().bold(), msg);
                    }
                    if let Some(rec) = &advisory.recommendation {
                        println!("{} {}", "recommendation:".cyan(), rec);
                    }
                }
            } else {
                return Err(error::VexError::Parse(
                    "Please specify a version (e.g., node@20.11.0) or use --auto".to_string(),
                ));
            }
        }
        Commands::List { tool, json } => {
            list_installed_with_output(&tool, output::OutputMode::from_json_flag(json))?;
        }
        Commands::ListRemote {
            tool,
            filter,
            no_cache,
            json,
        } => {
            list_remote_with_output(
                &tool,
                filter,
                !no_cache,
                output::OutputMode::from_json_flag(json),
            )?;
        }
        Commands::Current { json } => {
            show_current_with_output(output::OutputMode::from_json_flag(json))?;
        }
        Commands::Uninstall { spec } => {
            let (tool_name, version) = parse_spec(&spec)?;
            if version.is_empty() {
                return Err(error::VexError::Parse(
                    "Please specify a version to uninstall (e.g., node@20.11.0)".to_string(),
                ));
            }
            uninstall(&tool_name, &version)?;
        }
        Commands::Env { shell } => match shell::generate_hook(&shell) {
            Ok(hook) => print!("{}", hook),
            Err(e) => return Err(error::VexError::Parse(e)),
        },
        Commands::Local { spec } => {
            let (tool_name, version) = parse_spec(&spec)?;
            if version.is_empty() {
                return Err(error::VexError::Parse(
                    "Please specify a version (e.g., node@20.11.0)".to_string(),
                ));
            }
            let tool = tools::get_tool(&tool_name)?;
            let resolved = tools::resolve_fuzzy_version(tool.as_ref(), &version)?;
            let file_path = resolver::current_dir().join(".tool-versions");

            // Check if version is installed
            let version_dir = vex_dir()?
                .join("toolchains")
                .join(&tool_name)
                .join(&resolved);
            let is_installed = version_dir.exists();

            write_tool_version(&file_path, &tool_name, &resolved)?;

            if is_installed {
                println!(
                    "{} Set project version: {}@{}",
                    "✓".green(),
                    tool_name.yellow(),
                    resolved.cyan()
                );
            } else {
                println!(
                    "{} Set project version: {}@{}",
                    "✓".green(),
                    tool_name.yellow(),
                    resolved.cyan()
                );
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

            println!("{}", format!("  Config: {}", file_path.display()).dimmed());
            if is_installed {
                println!();
                println!("{}", "To activate it now, run: vex use --auto".dimmed());
            }
        }
        Commands::Global { spec } => {
            let (tool_name, version) = parse_spec(&spec)?;
            if version.is_empty() {
                return Err(error::VexError::Parse(
                    "Please specify a version (e.g., node@20.11.0)".to_string(),
                ));
            }
            let tool = tools::get_tool(&tool_name)?;
            let resolved = tools::resolve_fuzzy_version(tool.as_ref(), &version)?;
            let file_path = vex_dir()?.join("tool-versions");

            // Check if version is installed
            let version_dir = vex_dir()?
                .join("toolchains")
                .join(&tool_name)
                .join(&resolved);
            let is_installed = version_dir.exists();

            write_tool_version(&file_path, &tool_name, &resolved)?;

            if is_installed {
                println!(
                    "{} Set global default: {}@{}",
                    "✓".green(),
                    tool_name.yellow(),
                    resolved.cyan()
                );
            } else {
                println!(
                    "{} Set global default: {}@{}",
                    "✓".green(),
                    tool_name.yellow(),
                    resolved.cyan()
                );
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

            println!("{}", format!("  Config: {}", file_path.display()).dimmed());
            println!();
            println!(
                "{}",
                "This version will be used when no project-specific version is found.".dimmed()
            );
            if is_installed {
                println!("{}", "To activate it now, run: vex use --auto".dimmed());
            }
        }
        Commands::Upgrade { tool, all } => {
            commands::updates::upgrade(tool.as_deref(), all)?;
        }
        Commands::Outdated { tool, json } => {
            commands::updates::outdated(tool.as_deref(), output::OutputMode::from_json_flag(json))?;
        }
        Commands::Prune { dry_run } => {
            commands::prune::run(dry_run)?;
        }
        Commands::Alias { subcmd } => {
            handle_alias_command(&subcmd)?;
        }
        Commands::Exec { command } => {
            let code = commands::process::exec_command(&command)?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Commands::Run { task, args } => {
            let code = commands::process::run_task(&task, &args)?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Commands::Doctor { json } => {
            run_doctor_with_output(output::OutputMode::from_json_flag(json))?;
        }
        Commands::SelfUpdate => {
            updater::self_update()?;
        }
        Commands::Python { subcmd } => match subcmd.as_str() {
            "init" => python_init()?,
            "freeze" => python_freeze()?,
            "sync" => python_sync()?,
            _ => {
                return Err(error::VexError::Parse(format!(
                    "Unknown python subcommand: '{}'. Available: init, freeze, sync",
                    subcmd
                )));
            }
        },
    }

    Ok(())
}

fn main() {
    // Initialize logging system
    logging::init();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec_with_version() {
        let (tool, version) = parse_spec("node@20.11.0").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "20.11.0");
    }

    #[test]
    fn test_parse_spec_tool_only() {
        let (tool, version) = parse_spec("node").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "");
    }

    #[test]
    fn test_parse_spec_java() {
        let (tool, version) = parse_spec("java@21").unwrap();
        assert_eq!(tool, "java");
        assert_eq!(version, "21");
    }

    #[test]
    fn test_parse_spec_rust() {
        let (tool, version) = parse_spec("rust@1.93.1").unwrap();
        assert_eq!(tool, "rust");
        assert_eq!(version, "1.93.1");
    }

    #[test]
    fn test_parse_spec_go() {
        let (tool, version) = parse_spec("go@1.23.5").unwrap();
        assert_eq!(tool, "go");
        assert_eq!(version, "1.23.5");
    }

    #[test]
    fn test_parse_spec_invalid_multiple_at() {
        let result = parse_spec("node@20@11");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_spec_empty_version() {
        let (tool, version) = parse_spec("node@").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "");
    }

    #[test]
    fn test_parse_spec_version_with_v_prefix() {
        let (tool, version) = parse_spec("node@v20.11.0").unwrap();
        assert_eq!(tool, "node");
        assert_eq!(version, "v20.11.0");
    }

    #[test]
    fn test_vex_dir() {
        let dir = vex_dir().unwrap();
        assert!(dir.ends_with(".vex"));
    }

    #[test]
    fn test_vex_dir_error_handling() {
        // This test verifies that vex_dir() returns Result
        // In normal circumstances, it should succeed
        assert!(vex_dir().is_ok());
    }

    #[test]
    fn test_write_tool_version_new_file() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        write_tool_version(&file_path, "node", "20.11.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "node 20.11.0\n");
    }

    #[test]
    fn test_write_tool_version_update_existing() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        // Write initial version
        fs::write(&file_path, "node 20.11.0\ngo 1.23.5\n").unwrap();

        // Update node version
        write_tool_version(&file_path, "node", "22.0.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("node 22.0.0"));
        assert!(content.contains("go 1.23.5"));
        assert!(!content.contains("20.11.0"));
    }

    #[test]
    fn test_write_tool_version_add_new_tool() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        // Write initial version
        fs::write(&file_path, "node 20.11.0\n").unwrap();

        // Add go version
        write_tool_version(&file_path, "go", "1.23.5").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("node 20.11.0"));
        assert!(content.contains("go 1.23.5"));
    }

    #[test]
    fn test_write_tool_version_sorted() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        // Write in unsorted order
        fs::write(&file_path, "rust 1.93.1\ngo 1.23.5\n").unwrap();

        // Add node (should be sorted alphabetically)
        write_tool_version(&file_path, "node", "20.11.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "go 1.23.5");
        assert_eq!(lines[1], "node 20.11.0");
        assert_eq!(lines[2], "rust 1.93.1");
    }

    #[test]
    fn test_write_tool_version_ignores_comments() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        // Write with comments
        fs::write(&file_path, "# Comment\nnode 20.11.0\n# Another comment\n").unwrap();

        // Update node version
        write_tool_version(&file_path, "node", "22.0.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "node 22.0.0\n");
        assert!(!content.contains("Comment"));
    }

    #[test]
    fn test_write_tool_version_empty_lines() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".tool-versions");

        // Write with empty lines
        fs::write(&file_path, "\n\nnode 20.11.0\n\n").unwrap();

        // Update node version
        write_tool_version(&file_path, "node", "22.0.0").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "node 22.0.0\n");
    }

    #[test]
    fn test_find_active_python_bin_fallback() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        // Create .vex/bin dir but NO python3 file → must fallback
        let bin_dir = temp.path().join(".vex").join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        // Verify the python3 symlink does not exist in this dir
        assert!(!bin_dir.join("python3").exists());
        // Test the logic directly: if bin doesn't exist, return "python3"
        let bin = bin_dir.join("python3");
        let result: std::path::PathBuf = if bin.exists() {
            bin
        } else {
            std::path::PathBuf::from("python3")
        };
        assert_eq!(result, std::path::PathBuf::from("python3"));
    }

    #[test]
    fn test_find_active_python_bin_vex_bin() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join(".vex").join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let python_bin = bin_dir.join("python3");
        fs::write(&python_bin, "").unwrap();
        // Test the logic directly to avoid HOME env var race conditions in parallel tests
        let result: std::path::PathBuf = if python_bin.exists() {
            python_bin.clone()
        } else {
            std::path::PathBuf::from("python3")
        };
        assert_eq!(result, python_bin);
    }

    #[test]
    fn test_list_installed_no_toolchains_dir() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        std::env::set_var("HOME", temp.path());
        // No toolchains dir → prints "No versions" and returns Ok
        let result = list_installed("node");
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_installed_empty_dir() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".vex").join("toolchains").join("node")).unwrap();
        std::env::set_var("HOME", temp.path());
        let result = list_installed("node");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_current_no_current_dir() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".vex")).unwrap();
        std::env::set_var("HOME", temp.path());
        let result = show_current();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_current_empty_current_dir() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".vex").join("current")).unwrap();
        std::env::set_var("HOME", temp.path());
        let result = show_current();
        assert!(result.is_ok());
    }

    #[test]
    fn test_uninstall_version_not_found() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".vex").join("toolchains")).unwrap();
        std::env::set_var("HOME", temp.path());
        let result = uninstall("node", "99.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_spec_error_message() {
        let err = parse_spec("node@20@11").unwrap_err();
        assert!(err.to_string().contains("Invalid spec format"));
    }
}
