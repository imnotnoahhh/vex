use clap::Args;

#[derive(Args)]
pub(crate) struct PythonArgs {
    /// Subcommand:
    ///   init   — Create .venv in the current directory using the active vex-managed Python.
    ///            Also records the Python version in .tool-versions.
    ///   freeze — Run `pip freeze` and write output to requirements.lock.
    ///            Use after installing packages to lock the environment.
    ///   sync   — Restore the environment from requirements.lock via `pip install -r`.
    ///            Auto-creates .venv if it does not exist yet.
    pub(crate) subcmd: String,
}
