use super::{BIN_DIR, CACHE_DIR, CURRENT_DIR, TOOLCHAINS_DIR, VEX_DIR_NAME};
use std::path::PathBuf;

/// Get vex home directory path.
///
/// Returns `~/.vex` or the path specified by `VEX_HOME`.
pub fn vex_home() -> Option<PathBuf> {
    std::env::var("VEX_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|path| path.join(VEX_DIR_NAME)))
}

/// Get toolchains directory path.
pub fn toolchains_dir() -> Option<PathBuf> {
    vex_home().map(|path| path.join(TOOLCHAINS_DIR))
}

/// Get current version symlink directory path.
pub fn current_dir() -> Option<PathBuf> {
    vex_home().map(|path| path.join(CURRENT_DIR))
}

/// Get binary symlinks directory path.
pub fn bin_dir() -> Option<PathBuf> {
    vex_home().map(|path| path.join(BIN_DIR))
}

/// Get cache directory path.
pub fn cache_dir() -> Option<PathBuf> {
    vex_home().map(|path| path.join(CACHE_DIR))
}

pub fn config_path() -> Option<PathBuf> {
    vex_home().map(|path| path.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_directory_names() {
        assert_eq!(VEX_DIR_NAME, ".vex");
        assert_eq!(TOOLCHAINS_DIR, "toolchains");
        assert_eq!(CURRENT_DIR, "current");
        assert_eq!(BIN_DIR, "bin");
        assert_eq!(CACHE_DIR, "cache");
    }

    #[test]
    fn test_vex_home() {
        assert!(vex_home().is_some());
    }

    #[test]
    fn test_subdirectories() {
        if let Some(home) = vex_home() {
            assert_eq!(toolchains_dir(), Some(home.join(TOOLCHAINS_DIR)));
            assert_eq!(current_dir(), Some(home.join(CURRENT_DIR)));
            assert_eq!(bin_dir(), Some(home.join(BIN_DIR)));
            assert_eq!(cache_dir(), Some(home.join(CACHE_DIR)));
            assert_eq!(config_path(), Some(home.join("config.toml")));
        }
    }

    #[test]
    fn test_vex_home_with_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        let original = std::env::var("VEX_HOME").ok();

        std::env::set_var("VEX_HOME", "/tmp/test_vex");
        assert_eq!(vex_home(), Some(PathBuf::from("/tmp/test_vex")));

        if let Some(value) = original {
            std::env::set_var("VEX_HOME", value);
        } else {
            std::env::remove_var("VEX_HOME");
        }
    }
}
