use crate::error::{Result, VexError};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn base_root(vex_dir: &Path) -> PathBuf {
    vex_dir.join("python").join("base")
}

pub fn base_env_dir(vex_dir: &Path, version: &str) -> PathBuf {
    base_root(vex_dir).join(version)
}

pub fn base_bin_dir(vex_dir: &Path, version: &str) -> PathBuf {
    base_env_dir(vex_dir, version).join("bin")
}

pub fn base_python_bin(vex_dir: &Path, version: &str) -> PathBuf {
    base_bin_dir(vex_dir, version).join("python")
}

pub fn base_pip_bin(vex_dir: &Path, version: &str) -> PathBuf {
    base_bin_dir(vex_dir, version).join("pip")
}

pub fn is_base_env_healthy(vex_dir: &Path, version: &str) -> bool {
    base_python_bin(vex_dir, version).exists() && base_pip_bin(vex_dir, version).exists()
}

pub fn ensure_base_environment(
    vex_dir: &Path,
    version: &str,
    install_dir: &Path,
) -> Result<PathBuf> {
    let base_dir = base_env_dir(vex_dir, version);
    if is_base_env_healthy(vex_dir, version) {
        return Ok(base_dir);
    }

    if base_dir.exists() {
        return Err(VexError::PythonEnv(format!(
            "Python base environment exists but is incomplete: {}. Remove it and run 'vex python base' to recreate it.",
            base_dir.display()
        )));
    }

    fs::create_dir_all(base_root(vex_dir))?;
    let python = toolchain_python_bin(install_dir)?;
    let output = Command::new(&python)
        .args(["-m", "venv"])
        .arg(&base_dir)
        .output()?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&base_dir);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(VexError::PythonEnv(format!(
            "Failed to create Python base environment with {}: {}",
            python.display(),
            stderr.trim()
        )));
    }

    Ok(base_dir)
}

fn toolchain_python_bin(install_dir: &Path) -> Result<PathBuf> {
    for name in ["python3", "python"] {
        let candidate = install_dir.join("bin").join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(VexError::PythonEnv(format!(
        "No python binary found in {}",
        install_dir.join("bin").display()
    )))
}
