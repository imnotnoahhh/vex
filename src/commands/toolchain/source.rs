mod install;
mod summary;

use crate::error::Result;

pub use install::{install_from_source, install_specs, sync_from_source};

pub(super) fn sync_versions(versions: &[(String, String)], offline: bool) -> Result<()> {
    install::sync_versions(versions, offline)
}
