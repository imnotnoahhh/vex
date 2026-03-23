use super::{Tool, Version};
use crate::error::Result;

pub(super) fn fetch_versions_with_cache(tool: &dyn Tool, use_cache: bool) -> Result<Vec<Version>> {
    use crate::{cache, config};

    let vex = config::vex_home().ok_or(crate::error::VexError::HomeDirectoryNotFound)?;
    let remote_cache = cache::RemoteCache::new(&vex);
    let ttl = config::cache_ttl()?.as_secs();

    if use_cache {
        if let Some(cached) = remote_cache.get_cached_versions(tool.name(), ttl) {
            return Ok(cached);
        }
    }

    let versions = tool.list_remote()?;
    remote_cache.set_cached_versions(tool.name(), &versions);
    Ok(versions)
}
