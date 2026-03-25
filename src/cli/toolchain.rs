use clap::Args;

#[derive(Args)]
pub(crate) struct InstallArgs {
    /// Tool and version specs (e.g., node@20, go@1.22). Omit to install from .tool-versions.
    pub(crate) specs: Vec<String>,

    /// Skip automatic version switching after installation
    #[arg(long)]
    pub(crate) no_switch: bool,

    /// Force reinstall even if already installed
    #[arg(long)]
    pub(crate) force: bool,

    /// Install from a specific version source (version file, vex-config.toml, HTTPS URL, or Git repo)
    #[arg(long)]
    pub(crate) from: Option<String>,

    /// Frozen mode: strictly enforce lockfile versions, fail if lockfile is missing or versions don't match
    #[arg(long)]
    pub(crate) frozen: bool,

    /// Use offline mode (only use cached data, fail if unavailable)
    #[arg(long)]
    pub(crate) offline: bool,
}

#[derive(Args)]
pub(crate) struct SyncArgs {
    /// Install from a specific version source (version file, vex-config.toml, HTTPS URL, or Git repo)
    #[arg(long)]
    pub(crate) from: Option<String>,

    /// Frozen mode: strictly enforce lockfile versions, fail if lockfile is missing or versions don't match
    #[arg(long)]
    pub(crate) frozen: bool,

    /// Use offline mode (only use cached data, fail if unavailable)
    #[arg(long)]
    pub(crate) offline: bool,
}

#[derive(Args)]
pub(crate) struct UseArgs {
    /// Tool and version (e.g., node@20.11.0). Omit to auto-detect from version files.
    pub(crate) spec: Option<String>,

    /// Auto mode: read version files (.tool-versions, .node-version, etc.)
    #[arg(long)]
    pub(crate) auto: bool,
}

#[derive(Args)]
pub(crate) struct PinArgs {
    /// Tool and version (e.g., node@20.11.0)
    pub(crate) spec: String,
}
