use super::env::find_active_python_bin;
use crate::error::{Result, VexError};
use crate::resolver;
use crate::version_files;
use owo_colors::OwoColorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn init() -> Result<()> {
    let cwd = resolver::current_dir();
    let python_bin = find_active_python_bin()?;

    println!(
        "Creating .venv using {}...",
        python_bin.display().to_string().cyan()
    );

    let status = Command::new(&python_bin)
        .args(["-m", "venv", ".venv"])
        .current_dir(&cwd)
        .status()?;
    if !status.success() {
        return Err(VexError::Parse(
            "Failed to create .venv. Make sure python is installed via 'vex install python@<version>'".to_string(),
        ));
    }

    record_python_version(&cwd)?;
    println!(
        "{} Created .venv in {}",
        "✓".green(),
        cwd.display().to_string().dimmed()
    );
    println!();
    println!("{}", "To activate now:  source .venv/bin/activate".dimmed());
    println!(
        "{}",
        "Auto-activates:   next time you cd into this directory".dimmed()
    );

    Ok(())
}

pub(super) fn freeze() -> Result<()> {
    let cwd = resolver::current_dir();
    let pip = pip_path(&cwd);
    if !pip.exists() {
        return Err(VexError::PythonEnv(
            "No .venv found. Run 'vex python init' first.".to_string(),
        ));
    }

    let output = Command::new(&pip)
        .arg("freeze")
        .current_dir(&cwd)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(VexError::Parse(format!("pip freeze failed: {}", stderr)));
    }

    let lock_path = cwd.join("requirements.lock");
    fs::write(&lock_path, &output.stdout)?;
    let line_count = output.stdout.iter().filter(|&&byte| byte == b'\n').count();
    println!(
        "{} Wrote {} packages to {}",
        "✓".green(),
        line_count,
        "requirements.lock".cyan()
    );
    Ok(())
}

pub(super) fn sync() -> Result<()> {
    let cwd = resolver::current_dir();
    let venv = cwd.join(".venv");
    let lock_path = cwd.join("requirements.lock");

    if !lock_path.exists() {
        return Err(VexError::PythonEnv(
            "No requirements.lock found. Run 'vex python freeze' first.".to_string(),
        ));
    }

    if !venv.exists() {
        println!("{}", "No .venv found, initializing...".dimmed());
        init()?;
    }

    println!("Installing from requirements.lock...");
    let status = Command::new(pip_path(&cwd))
        .args(["install", "-r", "requirements.lock"])
        .current_dir(&cwd)
        .status()?;
    if !status.success() {
        return Err(VexError::Parse(
            "pip install failed. Check requirements.lock for errors.".to_string(),
        ));
    }

    println!(
        "{} Environment restored from requirements.lock",
        "✓".green()
    );
    Ok(())
}

fn record_python_version(cwd: &Path) -> Result<()> {
    let versions = resolver::resolve_versions(cwd);
    if let Some((_, version)) = versions.iter().find(|(tool, _)| tool.as_str() == "python") {
        let file_path = cwd.join(".tool-versions");
        version_files::write_tool_version(&file_path, "python", version)?;
        println!(
            "{} Recorded python {} in .tool-versions",
            "✓".green(),
            version.cyan()
        );
    }

    Ok(())
}

fn pip_path(cwd: &Path) -> PathBuf {
    cwd.join(".venv").join("bin").join("pip")
}
