use clap::Args;

#[derive(Args)]
pub(crate) struct EnvArgs {
    /// Shell type (zsh, bash, fish, or nu)
    pub(crate) shell: String,

    /// Output current managed exports for the active directory instead of the shell hook
    #[arg(long)]
    pub(crate) exports: bool,
}

#[derive(Args)]
pub(crate) struct ExecArgs {
    /// Command to run after '--' (for example: vex exec -- node -v)
    #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
    pub(crate) command: Vec<String>,
}

#[derive(Args)]
pub(crate) struct RunArgs {
    /// Task name from `[commands]` in `.vex.toml`
    pub(crate) task: String,

    /// Extra arguments appended to the task command
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub(crate) args: Vec<String>,
}
