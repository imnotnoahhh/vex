mod installed;
mod remote;
mod state;

use crate::error::Result;
use installed::collect_installed_versions;
use remote::collect_remote_versions;

use super::{InstalledVersionsReport, RemoteFilter, RemoteVersionsReport};

pub(super) fn collect_installed(tool_name: &str) -> Result<InstalledVersionsReport> {
    collect_installed_versions(tool_name)
}

pub(super) fn collect_remote(
    tool_name: &str,
    filter: RemoteFilter,
    use_cache: bool,
    offline: bool,
    show_spinner: bool,
) -> Result<RemoteVersionsReport> {
    collect_remote_versions(tool_name, filter, use_cache, offline, show_spinner)
}
