mod git;
mod remote;

use super::parse::validate_remote_team_config_response;
use super::{load_team_config, LoadedVersions, TEAM_CONFIG_FILE};
use crate::error::{Result, VexError};
use crate::resolver;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(super) enum SourceKind {
    VersionFile(PathBuf),
    TeamConfigFile(PathBuf),
    HttpsTeamConfig(String),
    GitRepo { source: String, is_local: bool },
}

pub(super) fn classify_source(source: &str, start_dir: &Path) -> Result<SourceKind> {
    if source.starts_with("https://") {
        if source.ends_with(".git") {
            return Ok(SourceKind::GitRepo {
                source: source.to_string(),
                is_local: false,
            });
        }
        return Ok(SourceKind::HttpsTeamConfig(source.to_string()));
    }

    if source.starts_with("git@") || source.starts_with("ssh://") {
        return Ok(SourceKind::GitRepo {
            source: source.to_string(),
            is_local: false,
        });
    }

    let path = resolve_source_path(source, start_dir);
    if path.is_dir() {
        if path.join(".git").exists() {
            return Ok(SourceKind::GitRepo {
                source: path.display().to_string(),
                is_local: true,
            });
        }
        return Err(VexError::Config(format!(
            "Directory source '{}' is not a Git repository.",
            path.display()
        )));
    }

    if path.is_file() {
        if is_team_config_path(&path) {
            return Ok(SourceKind::TeamConfigFile(path));
        }
        return Ok(SourceKind::VersionFile(path));
    }

    if is_team_config_path(&path) {
        return Ok(SourceKind::TeamConfigFile(path));
    }

    if source.ends_with(".git") {
        return Ok(SourceKind::GitRepo {
            source: source.to_string(),
            is_local: false,
        });
    }

    Ok(SourceKind::VersionFile(path))
}

pub(super) fn load_version_file(path: &Path) -> Result<LoadedVersions> {
    if !path.exists() {
        return Err(VexError::Config(format!(
            "Version file not found: {}",
            path.display()
        )));
    }

    let content = fs::read_to_string(path)?;
    Ok(LoadedVersions {
        description: path.display().to_string(),
        versions: resolver::parse_tool_versions(&content),
    })
}

pub(super) fn load_team_config_file(path: &Path, start_dir: &Path) -> Result<LoadedVersions> {
    if !path.exists() {
        return Err(VexError::Config(format!(
            "Team config file not found: {}",
            path.display()
        )));
    }

    let content = fs::read_to_string(path)?;
    load_team_config(&content, path.display().to_string(), start_dir)
}

pub(super) fn load_https_team_config(url: &str, start_dir: &Path) -> Result<LoadedVersions> {
    remote::load_https_team_config(url, start_dir)
}

pub(super) fn load_team_config_from_git_repo(source: &str, is_local: bool) -> Result<String> {
    git::load_team_config_from_git_repo(source, is_local)
}

pub(super) fn is_team_config_path(path: &Path) -> bool {
    path.file_name().and_then(|value| value.to_str()) == Some(TEAM_CONFIG_FILE)
}

fn resolve_source_path(source: &str, start_dir: &Path) -> PathBuf {
    let path = PathBuf::from(source);
    if path.is_absolute() {
        return path;
    }
    start_dir.join(path)
}
