mod alias;
mod init;
mod listing;
mod manage;
mod process;
mod python;
pub(crate) mod repair;
pub(crate) mod rust;
mod toolchain;

pub(crate) use alias::AliasCommands;
use clap::{Parser, Subcommand};

/// vex CLI main structure
#[derive(Parser)]
#[command(name = "vex", version)]
#[command(about = "A fast version manager for macOS", long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

/// CLI subcommand definitions
#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Initialize vex directory structure
    Init(init::InitArgs),

    /// Install a tool version (or all from .tool-versions)
    Install(toolchain::InstallArgs),

    /// Install missing versions from the current managed context
    Sync(toolchain::SyncArgs),

    /// Switch to a different version
    Use(toolchain::UseArgs),

    /// Rebuild managed binary links for the active toolchain
    Relink(manage::RelinkArgs),

    /// List installed versions
    List(listing::ListArgs),

    /// List available remote versions
    ListRemote(listing::ListRemoteArgs),

    /// Show current active versions
    Current(listing::CurrentArgs),

    /// Uninstall a version
    Uninstall(manage::UninstallArgs),

    /// Output shell hook for auto-switching
    Env(process::EnvArgs),

    /// Pin a tool version in the current directory (.tool-versions)
    Local(toolchain::PinArgs),

    /// Pin a tool version globally (~/.vex/tool-versions)
    Global(toolchain::PinArgs),

    /// Generate lockfile from current .tool-versions
    Lock,

    /// Upgrade a tool to the latest version
    Upgrade(manage::UpgradeArgs),

    /// Show which managed tools are behind the latest available version
    Outdated(manage::OutdatedArgs),

    /// Remove unused cache files, stale locks, and unreferenced toolchains
    #[command(alias = "gc")]
    Prune(manage::PruneArgs),

    /// Manage user-defined version aliases
    #[command(subcommand)]
    Alias(AliasCommands),

    /// Run a command inside the resolved vex-managed environment without switching global state
    Exec(process::ExecArgs),

    /// Run a named task from .vex.toml inside the resolved vex-managed environment
    Run(process::RunArgs),

    /// Check vex installation health
    Doctor(manage::DoctorArgs),

    /// Repair or migrate supported home-directory state into ~/.vex
    Repair(repair::RepairArgs),

    /// Update vex itself to the latest release
    SelfUpdate,

    /// Launch interactive TUI dashboard
    ///
    /// Shows current versions, health warnings, disk usage, and quick actions.
    /// Requires an interactive terminal.
    Tui,

    /// Python virtual environment management
    ///
    /// Workflow:
    ///   1. vex install python@3.12   (install a Python version globally)
    ///   2. cd my-project
    ///   3. vex python init            (create .venv using the active Python)
    ///   4. pip install `<packages>`
    ///   5. vex python freeze          (lock packages to requirements.lock)
    ///   6. vex python sync            (restore from requirements.lock on another machine)
    Python(python::PythonArgs),

    /// Official Rust toolchain extensions
    Rust(rust::RustArgs),
}
