use crate::error::Result;
use crate::paths::vex_dir;
use std::path::{Path, PathBuf};

pub(crate) fn find_active_python_bin() -> Result<PathBuf> {
    let vex = vex_dir()?;
    Ok(active_python_bin_in(&vex))
}

pub(super) fn active_python_bin_in(vex: &Path) -> PathBuf {
    let bin = vex.join("bin").join("python3");
    if bin.exists() {
        bin
    } else {
        PathBuf::from("python3")
    }
}
