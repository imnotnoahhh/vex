use super::super::super::types::{push_check, CheckStatus, DoctorCheck};
use super::strict_status;
use crate::config::{self, StrictMode};
use crate::home_state::{self, AuditKind};
use crate::tools::python;
use crate::version_state;
use std::path::Path;

pub(super) fn collect_home_hygiene_check(
    warnings: &mut usize,
    issues: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) {
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return,
    };
    let audits = home_state::audit(&home, Some("all"));
    if audits.is_empty() {
        push_check(
            checks,
            "home_hygiene",
            CheckStatus::Ok,
            "supported home-directory state is already contained in ~/.vex",
            Vec::new(),
        );
        return;
    }

    let mode = config::strict_home_hygiene().unwrap_or(StrictMode::Warn);
    let status = strict_status(mode, warnings, issues);
    let mut details = audits
        .iter()
        .map(|audit| match audit.kind {
            AuditKind::SafeMigration => format!(
                "{} -> {}",
                audit.source.display(),
                audit
                    .destination
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "~/.vex".to_string())
            ),
            AuditKind::Advisory => format!("{} ({})", audit.source.display(), audit.summary),
        })
        .collect::<Vec<_>>();
    details.push("Run 'vex repair migrate-home' to preview safe migrations.".to_string());
    push_check(
        checks,
        "home_hygiene",
        status,
        "legacy home-directory state was found outside ~/.vex",
        details,
    );
}

pub(super) fn collect_path_conflict_check(
    vex_bin: &Path,
    warnings: &mut usize,
    issues: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) {
    let path = std::env::var("PATH").unwrap_or_default();
    let vex_bin = vex_bin.to_string_lossy().to_string();
    let mut conflicts = Vec::new();
    for segment in path.split(':').filter(|segment| !segment.is_empty()) {
        if segment == vex_bin {
            break;
        }
        if segment.contains(".cargo/bin")
            || segment.ends_with("/go/bin")
            || segment.contains(".nvm")
            || segment.contains(".pyenv")
        {
            conflicts.push(segment.to_string());
        }
    }

    if conflicts.is_empty() {
        push_check(
            checks,
            "path_conflicts",
            CheckStatus::Ok,
            "PATH keeps ~/.vex entries ahead of common legacy manager bins",
            Vec::new(),
        );
        return;
    }

    let status = strict_status(
        config::strict_path_conflicts().unwrap_or(StrictMode::Warn),
        warnings,
        issues,
    );
    let mut details = conflicts;
    details.push("Keep ~/.vex/bin ahead of legacy manager bins in PATH.".to_string());
    push_check(
        checks,
        "path_conflicts",
        status,
        "PATH contains legacy manager bins ahead of ~/.vex/bin",
        details,
    );
}

pub(super) fn collect_captured_env_check(
    vex_dir: &Path,
    warnings: &mut usize,
    issues: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) {
    let expected_prefix = vex_dir.to_string_lossy().to_string();
    let mut mismatches = Vec::new();
    for key in [
        "CARGO_HOME",
        "GOPATH",
        "GOBIN",
        "GOMODCACHE",
        "GOCACHE",
        "GOENV",
        "NPM_CONFIG_CACHE",
        "NPM_CONFIG_PREFIX",
        "NPM_CONFIG_USERCONFIG",
        "COREPACK_HOME",
        "PNPM_HOME",
        "YARN_CACHE_FOLDER",
        "PIP_CACHE_DIR",
        "PYTHONUSERBASE",
    ] {
        if let Ok(value) = std::env::var(key) {
            if !value.starts_with(&expected_prefix) {
                mismatches.push(format!("{}={}", key, value));
            }
        }
    }

    if mismatches.is_empty() {
        push_check(
            checks,
            "captured_env",
            CheckStatus::Ok,
            "captured language home/cache variables point into ~/.vex",
            Vec::new(),
        );
        return;
    }

    let status = strict_status(
        config::strict_path_conflicts().unwrap_or(StrictMode::Warn),
        warnings,
        issues,
    );
    let mut details = mismatches;
    details.push("Re-open your shell with 'eval \"$(vex env <shell>)\"' and run 'vex repair migrate-home' if needed.".to_string());
    push_check(
        checks,
        "captured_env",
        status,
        "some captured language home/cache variables still point outside ~/.vex",
        details,
    );
}

pub(super) fn collect_manager_conflict_check(
    warnings: &mut usize,
    issues: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) {
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return,
    };
    let managers = [
        (".asdf", "asdf"),
        (".mise", "mise"),
        (".nvm", "nvm"),
        (".rustup", "rustup"),
        (".pyenv", "pyenv"),
    ];
    let present = managers
        .into_iter()
        .filter_map(|(path, label)| home.join(path).exists().then_some(label.to_string()))
        .collect::<Vec<_>>();

    if present.is_empty() {
        push_check(
            checks,
            "manager_conflicts",
            CheckStatus::Ok,
            "no common conflicting version manager homes were detected",
            Vec::new(),
        );
        return;
    }

    let status = strict_status(
        config::strict_path_conflicts().unwrap_or(StrictMode::Warn),
        warnings,
        issues,
    );
    let mut details = present;
    details.push(
        "These tools can coexist with vex, but they may still own files outside ~/.vex."
            .to_string(),
    );
    push_check(
        checks,
        "manager_conflicts",
        status,
        "other version-manager homes were detected",
        details,
    );
}

pub(super) fn collect_python_base_check(
    vex_dir: &Path,
    warnings: &mut usize,
    checks: &mut Vec<DoctorCheck>,
) {
    let current_versions = match version_state::read_current_versions(vex_dir) {
        Ok(versions) => versions,
        Err(_) => return,
    };
    let Some(version) = current_versions.get("python") else {
        push_check(
            checks,
            "python_base_env",
            CheckStatus::Ok,
            "python base environment check skipped because Python is not active",
            Vec::new(),
        );
        return;
    };

    let base_dir = python::base_env_dir(vex_dir, version);
    let base_bin = python::base_bin_dir(vex_dir, version);
    let mut details = vec![format!("Base: {}", base_dir.display())];
    let mut status = CheckStatus::Ok;
    let mut summary = "python base environment is ready".to_string();

    if !python::is_base_env_healthy(vex_dir, version) {
        *warnings += 1;
        status = CheckStatus::Warn;
        summary = "python base environment is missing or incomplete".to_string();
        details.push(format!(
            "Run 'vex python base' to create the base environment for python@{}.",
            version
        ));
    }

    if std::env::var_os("VIRTUAL_ENV").is_some() {
        let base_bin_str = base_bin.to_string_lossy().to_string();
        let leaks_into_venv = std::env::var("PATH")
            .unwrap_or_default()
            .split(':')
            .any(|entry| entry == base_bin_str);
        if leaks_into_venv {
            if status != CheckStatus::Warn {
                *warnings += 1;
            }
            status = CheckStatus::Warn;
            summary = "python base bin is active inside a virtual environment".to_string();
            details.push(
                "Project virtual environments should not inherit Python base CLI packages."
                    .to_string(),
            );
        }
    }

    push_check(checks, "python_base_env", status, &summary, details);
}
