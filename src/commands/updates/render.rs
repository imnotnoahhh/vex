mod outdated;
mod upgrade;

use super::ManagedSource;
pub(in crate::commands::updates) use outdated::render_outdated_text;
pub(in crate::commands::updates) use upgrade::render_upgrade_text;

fn source_label(source: ManagedSource) -> &'static str {
    match source {
        ManagedSource::Project => "project",
        ManagedSource::Global => "global",
        ManagedSource::Active => "active",
        ManagedSource::Installed => "installed",
    }
}
