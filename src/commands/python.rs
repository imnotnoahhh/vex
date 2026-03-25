mod env;
#[cfg(test)]
mod tests;
mod workflow;

use crate::error::{Result, VexError};

pub fn run_subcommand(subcmd: &str) -> Result<()> {
    match subcmd {
        "init" => init(),
        "freeze" => freeze(),
        "sync" => sync(),
        _ => Err(VexError::Parse(format!(
            "Unknown python subcommand: '{}'. Available: init, freeze, sync",
            subcmd
        ))),
    }
}

pub fn init() -> Result<()> {
    workflow::init()
}

pub fn freeze() -> Result<()> {
    workflow::freeze()
}

pub fn sync() -> Result<()> {
    workflow::sync()
}
