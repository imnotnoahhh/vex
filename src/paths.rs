use crate::config;
use crate::error::{Result, VexError};
use std::path::PathBuf;

pub fn vex_dir() -> Result<PathBuf> {
    config::vex_home().ok_or(VexError::HomeDirectoryNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vex_dir() {
        let dir = vex_dir().unwrap();
        assert!(dir.ends_with(".vex"));
    }

    #[test]
    fn test_vex_dir_error_handling() {
        assert!(vex_dir().is_ok());
    }
}
