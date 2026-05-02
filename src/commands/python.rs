mod env;
#[cfg(test)]
mod tests;
mod workflow;

use crate::error::{Result, VexError};

pub fn run_subcommand(subcmd: &str, args: &[String]) -> Result<()> {
    match subcmd {
        "init" => init(),
        "freeze" => freeze(),
        "sync" => sync(),
        "base" => base(args),
        _ => Err(VexError::Parse(format!(
            "Unknown python subcommand: '{}'. Available: init, freeze, sync, base",
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

pub fn base(args: &[String]) -> Result<()> {
    workflow::base(args)
}
