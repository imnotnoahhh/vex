use clap::{Args, Subcommand};

#[derive(Args)]
pub(crate) struct RustArgs {
    #[command(subcommand)]
    pub(crate) command: RustCommands,
}

#[derive(Subcommand)]
pub(crate) enum RustCommands {
    /// Manage official Rust targets
    Target(RustExtensionArgs),

    /// Manage official Rust components
    Component(RustExtensionArgs),
}

#[derive(Args)]
pub(crate) struct RustExtensionArgs {
    #[command(subcommand)]
    pub(crate) command: RustExtensionCommand,
}

#[derive(Subcommand)]
pub(crate) enum RustExtensionCommand {
    /// List targets or components for the active Rust toolchain
    List,

    /// Add one or more targets or components to the active Rust toolchain
    Add {
        #[arg(required = true)]
        names: Vec<String>,
    },

    /// Remove one or more managed targets or components from the active Rust toolchain
    Remove {
        #[arg(required = true)]
        names: Vec<String>,
    },
}
