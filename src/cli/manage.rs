use clap::Args;

#[derive(Args)]
pub(crate) struct UninstallArgs {
    /// Tool and version (e.g., node@20.11.0)
    pub(crate) spec: String,
}

#[derive(Args)]
pub(crate) struct UpgradeArgs {
    /// Tool name (e.g., node). Omit with --all.
    pub(crate) tool: Option<String>,

    /// Upgrade every managed tool in the current context
    #[arg(long)]
    pub(crate) all: bool,
}

#[derive(Args)]
pub(crate) struct OutdatedArgs {
    /// Tool name (e.g., node). Omit to inspect the current managed context.
    pub(crate) tool: Option<String>,

    /// Output machine-readable JSON
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Args)]
pub(crate) struct PruneArgs {
    /// Show what would be removed without deleting anything
    #[arg(long)]
    pub(crate) dry_run: bool,
}

#[derive(Args)]
pub(crate) struct DoctorArgs {
    /// Output machine-readable JSON
    #[arg(long)]
    pub(crate) json: bool,

    /// Show extended metadata and audit details
    #[arg(long)]
    pub(crate) verbose: bool,
}
