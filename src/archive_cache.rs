//! Archive cache module for reusable downloaded files
//!
//! Caches downloaded archives to `~/.vex/cache/archives/` to avoid re-downloading
//! when installing the same version multiple times.

use crate::checksum;
use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};
#[cfg(test)]
mod tests;

/// Default archive cache size cap. Beyond this, oldest archives are evicted (LRU by mtime).
pub const DEFAULT_CACHE_SIZE_LIMIT_BYTES: u64 = 5 * 1024 * 1024 * 1024; // 5 GB

/// Archive cache manager
///
/// Stores downloaded archives in `~/.vex/cache/archives/<tool>/<version>/<filename>`
pub struct ArchiveCache {
    cache_dir: PathBuf,
    size_limit_bytes: u64,
}

impl ArchiveCache {
    /// Create archive cache manager
    ///
    /// # Arguments
    /// - `vex_dir` - vex root directory (`~/.vex`)
    pub fn new(vex_dir: &Path) -> Self {
        Self::with_size_limit(vex_dir, DEFAULT_CACHE_SIZE_LIMIT_BYTES)
    }

    /// Create archive cache manager with a custom byte size cap.
    pub fn with_size_limit(vex_dir: &Path, size_limit_bytes: u64) -> Self {
        Self {
            cache_dir: vex_dir.join("cache").join("archives"),
            size_limit_bytes,
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

    fn checksum_path(&self, tool_name: &str, version: &str, filename: &str) -> PathBuf {
        self.tool_cache_dir(tool_name, version)
            .join(format!("{filename}.sha256"))
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

        if let Err(err) = self.enforce_size_limit() {
            warn!("Archive cache eviction failed: {}", err);
        }

        Ok(dest_path)
    }

    /// Evict oldest cached archives (by mtime) until total size is under the cap.
    /// Keeps the just-stored archive even if it alone exceeds the cap.
    pub fn enforce_size_limit(&self) -> Result<u64> {
        if self.size_limit_bytes == 0 || !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut entries = collect_cached_files(&self.cache_dir)?;
        let total: u64 = entries.iter().map(|e| e.size).sum();
        if total <= self.size_limit_bytes {
            return Ok(0);
        }

        // Oldest first, but keep the newest archive. If the newest archive alone
        // exceeds the cap, keeping it is better than deleting the file we just
        // downloaded and forcing the next install to download it again.
        entries.sort_by_key(|e| e.mtime);

        let mut evicted_bytes: u64 = 0;
        let mut running = total;
        let keep_index = entries.len().saturating_sub(1);
        for (idx, entry) in entries.into_iter().enumerate() {
            if running <= self.size_limit_bytes {
                break;
            }
            if idx == keep_index {
                break;
            }
            if remove_cached_archive(&entry.path).is_ok() {
                evicted_bytes = evicted_bytes.saturating_add(entry.size);
                running = running.saturating_sub(entry.size);
                debug!("Evicted cache archive: {}", entry.path.display());

                // Clean up empty per-version dirs to keep the layout tidy.
                if let Some(parent) = entry.path.parent() {
                    let _ = remove_dir_if_empty(parent);
                    if let Some(grandparent) = parent.parent() {
                        let _ = remove_dir_if_empty(grandparent);
                    }
                }
            }
        }

        if evicted_bytes > 0 {
            info!(
                "Pruned {} bytes from archive cache (cap {} bytes)",
                evicted_bytes, self.size_limit_bytes
            );
        }
        Ok(evicted_bytes)
    }

    /// Store verified archive checksum next to the cached archive.
    pub fn store_archive_checksum(
        &self,
        tool_name: &str,
        version: &str,
        filename: &str,
        checksum: &str,
    ) -> Result<()> {
        let cache_dir = self.tool_cache_dir(tool_name, version);
        fs::create_dir_all(&cache_dir)?;
        fs::write(
            self.checksum_path(tool_name, version, filename),
            format!("{}\n", checksum.trim()),
        )?;
        Ok(())
    }

    /// Get verified checksum recorded for a cached archive.
    pub fn get_archive_checksum(
        &self,
        tool_name: &str,
        version: &str,
        filename: &str,
    ) -> Result<Option<String>> {
        let path = self.checksum_path(tool_name, version, filename);
        if !path.exists() {
            return Ok(None);
        }

        Ok(Some(fs::read_to_string(path)?.trim().to_string()))
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

    /// Total bytes currently held by the archive cache.
    #[cfg(test)]
    pub fn total_size_bytes(&self) -> Result<u64> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }
        Ok(collect_cached_files(&self.cache_dir)?
            .iter()
            .map(|e| e.size)
            .sum())
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

struct CachedFile {
    path: PathBuf,
    size: u64,
    mtime: SystemTime,
}

fn collect_cached_files(root: &Path) -> Result<Vec<CachedFile>> {
    let mut out = Vec::new();
    walk_cache_files(root, &mut out)?;
    Ok(out)
}

fn walk_cache_files(dir: &Path, out: &mut Vec<CachedFile>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk_cache_files(&entry.path(), out)?;
        } else if file_type.is_file() {
            // Ignore companion checksum files when computing the cache footprint —
            // they're tiny and we evict them implicitly when the archive goes.
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|name| name.ends_with(".sha256"))
                .unwrap_or(false)
            {
                continue;
            }
            let meta = entry.metadata()?;
            let size = meta.len();
            let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            out.push(CachedFile { path, size, mtime });
        }
    }
    Ok(())
}

fn remove_dir_if_empty(path: &Path) -> std::io::Result<()> {
    if path.is_dir() && fs::read_dir(path)?.next().is_none() {
        fs::remove_dir(path)?;
    }
    Ok(())
}

fn remove_cached_archive(path: &Path) -> std::io::Result<()> {
    fs::remove_file(path)?;
    let checksum_path = path.with_file_name(format!(
        "{}.sha256",
        path.file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_default()
    ));
    if checksum_path.exists() {
        fs::remove_file(checksum_path)?;
    }
    Ok(())
}
