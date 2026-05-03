use super::env::find_active_python_bin;
use crate::error::{Result, VexError};
use crate::paths::vex_dir;
use crate::resolver;
use crate::tools::python;
use crate::version_files;
use owo_colors::OwoColorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

pub(super) fn base(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        None | Some("ensure") => {
            let (version, base_dir) = ensure_active_base()?;
            println!(
                "{} Python base environment is ready for python@{}",
                "✓".green(),
                version.cyan()
            );
            println!("{}", format!("  Base: {}", base_dir.display()).dimmed());
            println!(
                "{}",
                "  Use it for global Python CLIs when no project .venv is active.".dimmed()
            );
            Ok(())
        }
        Some("path") => {
            let (_version, base_dir) = ensure_active_base()?;
            println!("{}", base_dir.display());
            Ok(())
        }
        Some("pip") => run_base_pip(&args[1..]),
        Some("freeze") => freeze_base(),
        Some("sync") => sync_base(),
        Some(other) => Err(VexError::Parse(format!(
            "Unknown python base subcommand: '{}'. Available: ensure, path, pip, freeze, sync",
            other
        ))),
    }
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

fn ensure_active_base() -> Result<(String, PathBuf)> {
    let vex = vex_dir()?;
    let (version, install_dir) = active_python_install(&vex)?;
    let base_dir = python::ensure_base_environment(&vex, &version, &install_dir)?;
    Ok((version, base_dir))
}

fn active_python_install(vex: &Path) -> Result<(String, PathBuf)> {
    let current_link = vex.join("current").join("python");
    let target = fs::read_link(&current_link).map_err(|_| {
        VexError::PythonEnv(
            "No active vex-managed Python found. Run 'vex use python@<version>' first.".to_string(),
        )
    })?;
    let install_dir = if target.is_absolute() {
        target
    } else {
        current_link.parent().unwrap_or(vex).join(target)
    };
    let version = install_dir
        .file_name()
        .ok_or_else(|| {
            VexError::PythonEnv(format!(
                "Could not determine Python version from {}",
                install_dir.display()
            ))
        })?
        .to_string_lossy()
        .to_string();

    Ok((version, install_dir))
}

fn run_base_pip(args: &[String]) -> Result<()> {
    let vex = vex_dir()?;
    let (version, _base_dir) = ensure_active_base()?;
    if args.is_empty() {
        return Err(VexError::Parse(
            "Usage: vex python base pip <pip-args...>".to_string(),
        ));
    }

    let pip = python::base_pip_bin(&vex, &version);
    let mut command = Command::new(&pip);
    configure_base_pip_env(&mut command, &vex);
    let status = command
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(VexError::PythonEnv(format!(
            "Base pip command failed for python@{}",
            version
        )));
    }

    Ok(())
}

fn freeze_base() -> Result<()> {
    let vex = vex_dir()?;
    let (version, base_dir) = ensure_active_base()?;
    let pip = python::base_pip_bin(&vex, &version);
    let mut command = Command::new(&pip);
    configure_base_pip_env(&mut command, &vex);
    let output = command.arg("freeze").output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(VexError::PythonEnv(format!(
            "Base pip freeze failed for python@{}: {}",
            version,
            stderr.trim()
        )));
    }

    let lock_path = base_dir.join("requirements.lock");
    fs::write(&lock_path, &output.stdout)?;
    let line_count = output.stdout.iter().filter(|&&byte| byte == b'\n').count();
    println!(
        "{} Wrote {} base packages to {}",
        "✓".green(),
        line_count,
        lock_path.display().to_string().cyan()
    );
    Ok(())
}

fn sync_base() -> Result<()> {
    let vex = vex_dir()?;
    let (version, base_dir) = ensure_active_base()?;
    let lock_path = base_dir.join("requirements.lock");
    if !lock_path.exists() {
        return Err(VexError::PythonEnv(format!(
            "No base requirements.lock found for python@{}. Run 'vex python base freeze' first.",
            version
        )));
    }

    let pip = python::base_pip_bin(&vex, &version);
    let mut command = Command::new(&pip);
    configure_base_pip_env(&mut command, &vex);
    let status = command.arg("install").arg("-r").arg(&lock_path).status()?;
    if !status.success() {
        return Err(VexError::PythonEnv(format!(
            "Base pip sync failed for python@{}",
            version
        )));
    }

    println!(
        "{} Python base environment restored from {}",
        "✓".green(),
        lock_path.display().to_string().cyan()
    );
    Ok(())
}

fn configure_base_pip_env(command: &mut Command, vex: &Path) {
    command
        .env("PIP_CACHE_DIR", vex.join("pip/cache"))
        .env("PYTHONUSERBASE", python::user_base_dir(vex));
}
