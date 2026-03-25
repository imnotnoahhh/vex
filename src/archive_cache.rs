//! Archive cache module for reusable downloaded files
//!
//! Caches downloaded archives to `~/.vex/cache/archives/` to avoid re-downloading
//! when installing the same version multiple times.

use crate::checksum;
use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
#[cfg(test)]
mod tests;

/// Archive cache manager
///
/// Stores downloaded archives in `~/.vex/cache/archives/<tool>/<version>/<filename>`
pub struct ArchiveCache {
    cache_dir: PathBuf,
}

impl ArchiveCache {
    /// Create archive cache manager
    ///
    /// # Arguments
    /// - `vex_dir` - vex root directory (`~/.vex`)
    pub fn new(vex_dir: &Path) -> Self {
        Self {
            cache_dir: vex_dir.join("cache").join("archives"),
        }
    }

    /// Get cache path for a specific tool version
    fn tool_cache_dir(&self, tool_name: &str, version: &str) -> PathBuf {
        self.cache_dir.join(tool_name).join(version)
    }

    /// Get cache path for a specific archive file
    fn archive_path(&self, tool_name: &str, version: &str, filename: &str) -> PathBuf {
        self.tool_cache_dir(tool_name, version).join(filename)
    }

    /// Check if an archive exists in cache
    #[cfg(test)]
    pub fn has_archive(&self, tool_name: &str, version: &str, filename: &str) -> bool {
        let path = self.archive_path(tool_name, version, filename);
        path.exists() && path.is_file()
    }

    /// Get cached archive path if it exists
    pub fn get_archive(&self, tool_name: &str, version: &str, filename: &str) -> Option<PathBuf> {
        let path = self.archive_path(tool_name, version, filename);
        if path.exists() && path.is_file() {
            debug!("Archive cache hit: {}/{}", tool_name, version);
            Some(path)
        } else {
            debug!("Archive cache miss: {}/{}", tool_name, version);
            None
        }
    }

    /// Store an archive in cache
    pub fn store_archive(
        &self,
        tool_name: &str,
        version: &str,
        filename: &str,
        source_path: &Path,
    ) -> Result<PathBuf> {
        let cache_dir = self.tool_cache_dir(tool_name, version);
        fs::create_dir_all(&cache_dir)?;

        let dest_path = cache_dir.join(filename);
        fs::copy(source_path, &dest_path)?;

        info!(
            "Stored archive in cache: {}/{} -> {}",
            tool_name,
            version,
            dest_path.display()
        );

        Ok(dest_path)
    }

    /// Verify archive checksum
    pub fn verify_checksum(&self, archive_path: &Path, expected_checksum: &str) -> Result<()> {
        checksum::verify_sha256(archive_path, expected_checksum)
    }

    /// Clean up cache for a specific tool version
    #[cfg(test)]
    pub fn remove_version(&self, tool_name: &str, version: &str) -> Result<()> {
        let cache_dir = self.tool_cache_dir(tool_name, version);
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)?;
            info!("Removed archive cache: {}/{}", tool_name, version);
        }
        Ok(())
    }

    /// List all cached versions for a tool
    #[cfg(test)]
    pub fn list_cached_versions(&self, tool_name: &str) -> Result<Vec<String>> {
        let tool_dir = self.cache_dir.join(tool_name);
        if !tool_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        for entry in fs::read_dir(&tool_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    versions.push(name.to_string());
                }
            }
        }

        Ok(versions)
    }
}
