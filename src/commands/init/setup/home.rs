use crate::error::Result;
use std::fs;
use std::path::Path;

pub(super) fn initialize_vex_home(vex_dir: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        return Ok(());
    }

    for subdir in [
        "cache",
        "locks",
        "toolchains",
        "current",
        "bin",
        "go/bin",
        "go/pkg/mod",
        "go/cache",
        "npm/prefix/bin",
        "python/user/bin",
    ] {
        fs::create_dir_all(vex_dir.join(subdir))?;
    }

    let config_path = vex_dir.join("config.toml");
    if !config_path.exists() {
        fs::write(&config_path, "# vex configuration\n")?;
    }

    Ok(())
}
