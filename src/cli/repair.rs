use clap::{Args, Subcommand};

#[derive(Args)]
pub(crate) struct RepairArgs {
    #[command(subcommand)]
    pub(crate) command: RepairCommands,
}

#[derive(Subcommand)]
pub(crate) enum RepairCommands {
    /// Audit and migrate supported legacy home directories into ~/.vex
    MigrateHome(MigrateHomeArgs),
}

#[derive(Args)]
pub(crate) struct MigrateHomeArgs {
    /// Restrict the migration to a single tool (rust, go, node, python) or all
    #[arg(long)]
    pub(crate) tool: Option<String>,

    /// Apply the migration instead of showing a dry-run preview
    #[arg(long)]
    pub(crate) apply: bool,
}
