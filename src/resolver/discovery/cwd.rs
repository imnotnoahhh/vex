use std::env;
use std::path::PathBuf;

pub(super) fn current_dir() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
