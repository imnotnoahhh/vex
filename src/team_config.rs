mod parse;
mod source;

use crate::error::{Result, VexError};
use crate::resolver;
use parse::parse_team_config;
use source::{
    classify_source, load_https_team_config, load_team_config_file, load_team_config_from_git_repo,
    load_version_file, SourceKind,
};
use std::collections::BTreeMap;
use std::path::Path;

pub(super) const TEAM_CONFIG_FILE: &str = "vex-config.toml";

#[derive(Debug, Clone)]
pub struct LoadedVersions {
    pub description: String,
    pub versions: Vec<(String, String)>,
}

pub fn load_versions_from_source(
    source: &str,
    start_dir: &Path,
    offline: bool,
) -> Result<LoadedVersions> {
    let source_kind = classify_source(source, start_dir)?;
    match source_kind {
        SourceKind::VersionFile(path) => load_version_file(&path),
        SourceKind::TeamConfigFile(path) => load_team_config_file(&path, start_dir),
        SourceKind::HttpsTeamConfig(url) => {
            if offline {
                return Err(VexError::OfflineModeError(
                    "Remote team config requires network access. Re-run without --offline."
                        .to_string(),
                ));
            }
            load_https_team_config(&url, start_dir)
        }
        SourceKind::GitRepo { source, is_local } => {
            if offline && !is_local {
                return Err(VexError::OfflineModeError(
                    "Remote Git team config requires network access. Re-run without --offline."
                        .to_string(),
                ));
            }
            let content = load_team_config_from_git_repo(&source, is_local)?;
            load_team_config(&content, source, start_dir)
        }
    }
}

fn load_team_config(
    content: &str,
    description: String,
    start_dir: &Path,
) -> Result<LoadedVersions> {
    let baseline = parse_team_config(content)?;
    let overrides = resolver::resolve_local_tool_versions_only(start_dir);

    let mut merged: BTreeMap<String, String> = baseline.into_iter().collect();
    for (tool, version) in overrides {
        merged.insert(tool, version);
    }

    Ok(LoadedVersions {
        description,
        versions: merged.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests;
