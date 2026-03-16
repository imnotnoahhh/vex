//! Archive cache module for reusable downloaded files
//!
//! Caches downloaded archives to `~/.vex/cache/archives/` to avoid re-downloading
//! when installing the same version multiple times.

use crate::error::{Result, VexError};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

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
    ///
    /// # Arguments
    /// - `tool_name` - Tool name
    /// - `version` - Version string
    /// - `filename` - Archive filename
    #[allow(dead_code)]
    pub fn has_archive(&self, tool_name: &str, version: &str, filename: &str) -> bool {
        let path = self.archive_path(tool_name, version, filename);
        path.exists() && path.is_file()
    }

    /// Get cached archive path if it exists
    ///
    /// # Arguments
    /// - `tool_name` - Tool name
    /// - `version` - Version string
    /// - `filename` - Archive filename
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
    ///
    /// # Arguments
    /// - `tool_name` - Tool name
    /// - `version` - Version string
    /// - `filename` - Archive filename
    /// - `source_path` - Path to the downloaded archive
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

        // Copy file to cache
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
    ///
    /// # Arguments
    /// - `archive_path` - Path to archive file
    /// - `expected_checksum` - Expected SHA256 checksum (hex string)
    pub fn verify_checksum(&self, archive_path: &Path, expected_checksum: &str) -> Result<()> {
        let mut file = fs::File::open(archive_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let computed = format!("{:x}", hasher.finalize());

        if computed != expected_checksum {
            return Err(VexError::ChecksumMismatch {
                expected: expected_checksum.to_string(),
                actual: computed,
            });
        }

        Ok(())
    }

    /// Clean up cache for a specific tool version
    ///
    /// # Arguments
    /// - `tool_name` - Tool name
    /// - `version` - Version string
    #[allow(dead_code)]
    pub fn remove_version(&self, tool_name: &str, version: &str) -> Result<()> {
        let cache_dir = self.tool_cache_dir(tool_name, version);
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)?;
            info!("Removed archive cache: {}/{}", tool_name, version);
        }
        Ok(())
    }

    /// List all cached versions for a tool
    ///
    /// # Arguments
    /// - `tool_name` - Tool name
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_archive_cache_miss() {
        let tmp = TempDir::new().unwrap();
        let cache = ArchiveCache::new(tmp.path());

        assert!(!cache.has_archive("node", "20.11.0", "node-v20.11.0.tar.gz"));
        assert!(cache
            .get_archive("node", "20.11.0", "node-v20.11.0.tar.gz")
            .is_none());
    }

    #[test]
    fn test_archive_cache_store_and_retrieve() {
        let tmp = TempDir::new().unwrap();
        let cache = ArchiveCache::new(tmp.path());

        // Create a test file
        let test_file = tmp.path().join("test.tar.gz");
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        // Store in cache
        let cached_path = cache
            .store_archive("node", "20.11.0", "node-v20.11.0.tar.gz", &test_file)
            .unwrap();

        // Verify it exists
        assert!(cache.has_archive("node", "20.11.0", "node-v20.11.0.tar.gz"));

        // Retrieve from cache
        let retrieved = cache
            .get_archive("node", "20.11.0", "node-v20.11.0.tar.gz")
            .unwrap();
        assert_eq!(retrieved, cached_path);

        // Verify content
        let content = fs::read_to_string(&retrieved).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_archive_cache_remove_version() {
        let tmp = TempDir::new().unwrap();
        let cache = ArchiveCache::new(tmp.path());

        // Create and store a test file
        let test_file = tmp.path().join("test.tar.gz");
        fs::write(&test_file, b"test").unwrap();
        cache
            .store_archive("node", "20.11.0", "node-v20.11.0.tar.gz", &test_file)
            .unwrap();

        assert!(cache.has_archive("node", "20.11.0", "node-v20.11.0.tar.gz"));

        // Remove version
        cache.remove_version("node", "20.11.0").unwrap();

        assert!(!cache.has_archive("node", "20.11.0", "node-v20.11.0.tar.gz"));
    }

    #[test]
    fn test_list_cached_versions() {
        let tmp = TempDir::new().unwrap();
        let cache = ArchiveCache::new(tmp.path());

        // Initially empty
        assert_eq!(cache.list_cached_versions("node").unwrap().len(), 0);

        // Store multiple versions
        let test_file = tmp.path().join("test.tar.gz");
        fs::write(&test_file, b"test").unwrap();

        cache
            .store_archive("node", "20.11.0", "node-v20.11.0.tar.gz", &test_file)
            .unwrap();
        cache
            .store_archive("node", "22.0.0", "node-v22.0.0.tar.gz", &test_file)
            .unwrap();

        let versions = cache.list_cached_versions("node").unwrap();
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"20.11.0".to_string()));
        assert!(versions.contains(&"22.0.0".to_string()));
    }

    #[test]
    fn test_verify_checksum_success() {
        let tmp = TempDir::new().unwrap();
        let cache = ArchiveCache::new(tmp.path());

        // Create a test file with known content
        let test_file = tmp.path().join("test.tar.gz");
        fs::write(&test_file, b"test content").unwrap();

        // Calculate expected checksum
        let mut hasher = Sha256::new();
        hasher.update(b"test content");
        let expected = format!("{:x}", hasher.finalize());

        // Verify should succeed
        assert!(cache.verify_checksum(&test_file, &expected).is_ok());
    }

    #[test]
    fn test_verify_checksum_failure() {
        let tmp = TempDir::new().unwrap();
        let cache = ArchiveCache::new(tmp.path());

        let test_file = tmp.path().join("test.tar.gz");
        fs::write(&test_file, b"test content").unwrap();

        // Use wrong checksum
        let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";

        assert!(cache.verify_checksum(&test_file, wrong_checksum).is_err());
    }
}
