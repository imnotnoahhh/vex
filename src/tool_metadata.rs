use crate::error::Result;
use crate::tools::Tool;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const METADATA_FILE_NAME: &str = ".vex-metadata.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolchainMetadata {
    pub tool: String,
    pub version: String,
    pub provenance: ProvenanceMetadata,
    pub ownership: OwnershipMetadata,
    #[serde(default)]
    pub extensions: Vec<ExtensionMetadata>,
    #[serde(default)]
    pub managed_env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvenanceMetadata {
    pub source_url: Option<String>,
    pub mirror_url: Option<String>,
    pub checksum: Option<String>,
    pub installed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OwnershipMetadata {
    #[serde(default)]
    pub vex_owned: Vec<String>,
    #[serde(default)]
    pub managed_user_state: Vec<String>,
    #[serde(default)]
    pub project_owned: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtensionMetadata {
    pub kind: String,
    pub name: String,
    pub source_url: Option<String>,
    pub checksum: Option<String>,
    pub installed_at: String,
    #[serde(default)]
    pub owned_paths: Vec<String>,
}

pub fn metadata_path(install_dir: &Path) -> PathBuf {
    install_dir.join(METADATA_FILE_NAME)
}

pub fn read_metadata(install_dir: &Path) -> Result<Option<ToolchainMetadata>> {
    let path = metadata_path(install_dir);
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;
    let metadata = serde_json::from_str(&content)
        .map_err(|err| crate::error::VexError::Parse(format!("Invalid tool metadata: {}", err)))?;
    Ok(Some(metadata))
}

pub fn write_metadata(install_dir: &Path, metadata: &ToolchainMetadata) -> Result<()> {
    let path = metadata_path(install_dir);
    let json = serde_json::to_string_pretty(metadata)
        .map_err(|err| crate::error::VexError::Parse(format!("Invalid tool metadata: {}", err)))?;
    fs::write(path, json)?;
    Ok(())
}

pub fn write_base_metadata(
    tool: &dyn Tool,
    version: &str,
    install_dir: &Path,
    vex_dir: &Path,
    source_url: &str,
    mirror_url: Option<&str>,
    checksum: Option<&str>,
) -> Result<()> {
    let managed = tool.managed_environment(vex_dir, Some(install_dir));
    let metadata = ToolchainMetadata {
        tool: tool.name().to_string(),
        version: version.to_string(),
        provenance: ProvenanceMetadata {
            source_url: Some(source_url.to_string()),
            mirror_url: mirror_url.map(ToString::to_string),
            checksum: checksum.map(ToString::to_string),
            installed_at: Utc::now().to_rfc3339(),
        },
        ownership: OwnershipMetadata {
            vex_owned: vec![install_dir.display().to_string()],
            managed_user_state: managed.owned_home_dirs,
            project_owned: managed.project_owned_dirs,
        },
        extensions: Vec::new(),
        managed_env: managed.managed_env,
    };

    write_metadata(install_dir, &metadata)
}
