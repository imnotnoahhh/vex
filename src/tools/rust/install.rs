use crate::error::Result;
use crate::tools::Arch;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use super::dist::target_triple;

pub(crate) fn link_runtime_components(install_dir: &Path, arch: Arch) -> Result<()> {
    link_standard_library_component(install_dir, target_triple(arch))?;

    for component in &["clippy-preview", "rustfmt-preview", "rust-analyzer-preview"] {
        link_preview_lib_component(install_dir, component)?;
    }

    Ok(())
}

pub(crate) fn link_standard_library_component(install_dir: &Path, target: &str) -> Result<()> {
    let std_src = install_dir
        .join(format!("rust-std-{}", target))
        .join("lib/rustlib")
        .join(target)
        .join("lib");
    let std_dst = install_dir
        .join("rustc/lib/rustlib")
        .join(target)
        .join("lib");
    ensure_symlink(&std_src, &std_dst)
}

pub(crate) fn link_rust_src_component(install_dir: &Path) -> Result<()> {
    let src = install_dir.join("rust-src/lib/rustlib/src");
    let dst = install_dir.join("rustc/lib/rustlib/src");
    ensure_symlink(&src, &dst)
}

pub(crate) fn link_preview_lib_component(install_dir: &Path, component: &str) -> Result<()> {
    let rustc_lib = install_dir.join("rustc/lib");
    let lib_link = install_dir.join(component).join("lib");
    ensure_symlink(&rustc_lib, &lib_link)
}

pub(crate) fn remove_component_link(path: &Path) -> Result<()> {
    if path
        .symlink_metadata()
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
    {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn ensure_symlink(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() || destination.exists() {
        return Ok(());
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    unix_fs::symlink(relative_link_target(source, destination), destination)?;
    Ok(())
}

fn relative_link_target(source: &Path, destination: &Path) -> PathBuf {
    let source_components = source.components().collect::<Vec<_>>();
    let destination_components = destination
        .parent()
        .unwrap_or(destination)
        .components()
        .collect::<Vec<_>>();

    let mut common = 0usize;
    while common < source_components.len()
        && common < destination_components.len()
        && source_components[common] == destination_components[common]
    {
        common += 1;
    }

    let mut relative = PathBuf::new();
    for _ in common..destination_components.len() {
        relative.push("..");
    }
    for component in &source_components[common..] {
        relative.push(component.as_os_str());
    }
    relative
}
