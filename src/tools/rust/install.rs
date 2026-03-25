use crate::error::Result;
use crate::tools::Arch;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use super::dist::target_triple;

pub(super) fn link_runtime_components(install_dir: &Path, arch: Arch) -> Result<()> {
    let target = target_triple(arch);

    let std_src = install_dir
        .join(format!("rust-std-{}", target))
        .join("lib/rustlib")
        .join(target)
        .join("lib");
    let std_dst = install_dir
        .join("rustc/lib/rustlib")
        .join(target)
        .join("lib");
    if std_src.exists() && !std_dst.exists() {
        unix_fs::symlink(&std_src, &std_dst)?;
    }

    let rustc_lib = install_dir.join("rustc/lib");
    for component in &["clippy-preview", "rustfmt-preview", "rust-analyzer-preview"] {
        let lib_link = install_dir.join(component).join("lib");
        if rustc_lib.exists() && !lib_link.exists() {
            unix_fs::symlink(&rustc_lib, &lib_link)?;
        }
    }

    Ok(())
}
