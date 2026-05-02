use clap::Args;

#[derive(Args)]
pub(crate) struct ListArgs {
    /// Tool name (e.g., node)
    pub(crate) tool: String,

    /// Output machine-readable JSON
    #[arg(long)]
    pub(crate) json: bool,

    /// Show extended metadata details in text output
    #[arg(long)]
    pub(crate) verbose: bool,
}

#[derive(Args)]
pub(crate) struct ListRemoteArgs {
    /// Tool name (e.g., node)
    pub(crate) tool: String,

    /// Filter type (all, lts, major, latest)
    #[arg(long, short = 'f', default_value = "all")]
    pub(crate) filter: crate::commands::versions::RemoteFilter,

    /// Skip cache and fetch fresh data
    #[arg(long)]
    pub(crate) no_cache: bool,

    /// Use offline mode (only use cached data, fail if unavailable)
    #[arg(long)]
    pub(crate) offline: bool,

    /// Output machine-readable JSON
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Args)]
pub(crate) struct CurrentArgs {
    /// Output machine-readable JSON
    #[arg(long)]
    pub(crate) json: bool,

    /// Show extended metadata details in text output
    #[arg(long)]
    pub(crate) verbose: bool,
}

#[derive(Args)]
pub(crate) struct GlobalsArgs {
    /// Optional tool/ecosystem filter: node, python, go, rust, java, maven, or gradle
    pub(crate) tool: Option<String>,

    /// Output machine-readable JSON
    #[arg(long)]
    pub(crate) json: bool,

    /// Show full executable and source paths in text output
    #[arg(long)]
    pub(crate) verbose: bool,
}
