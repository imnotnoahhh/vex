use clap::Subcommand;

/// Alias subcommands
#[derive(Subcommand)]
pub(crate) enum AliasCommands {
    /// Set an alias for a tool version
    Set {
        /// Tool name (e.g., node)
        tool: String,

        /// Alias name (e.g., prod, dev, ci)
        alias: String,

        /// Version to alias (e.g., 20.11.0)
        version: String,

        /// Set as project alias (.vex.toml) instead of global
        #[arg(long)]
        project: bool,
    },

    /// List aliases
    List {
        /// Tool name to filter (optional)
        tool: Option<String>,

        /// Show only project aliases
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

        /// Alias name to delete
        alias: String,

        /// Delete from project aliases instead of global
        #[arg(long)]
        project: bool,
    },
}
