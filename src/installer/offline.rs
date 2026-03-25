use super::extract::{extract_archive, find_extracted_root};
use super::support::CleanupGuard;
use crate::archive_cache::ArchiveCache;
use crate::config;
use crate::error::{Result, VexError};
use crate::lock::InstallLock;
use crate::paths::vex_dir;
use crate::tools::{Arch, Tool};
use crate::ui;
use flate2::read::GzDecoder;
use owo_colors::OwoColorize;
use std::fs;
use tar::Archive;
use tracing::{debug, info};

pub(super) fn install_offline(tool: &dyn Tool, version: &str) -> Result<()> {
    info!("Starting offline installation: {}@{}", tool.name(), version);
    let arch = Arch::detect()?;
    let vex = vex_dir()?;

    let final_dir = config::toolchains_dir()
        .ok_or(VexError::HomeDirectoryNotFound)?
        .join(tool.name())
        .join(version);
    if final_dir.exists() {
        info!("Version already installed: {}@{}", tool.name(), version);
        ui::success(&format!(
            "{} is already installed.",
            format!("{}@{}", tool.name(), version).yellow()
        ));
        return Ok(());
    }

    let _lock = InstallLock::acquire(&vex, tool.name(), version)?;

    let archive_cache = ArchiveCache::new(&vex);
    let archive_name = format!("{}-{}.tar.gz", tool.name(), version);
    let cached_archive = archive_cache
        .get_archive(tool.name(), version, &archive_name)
        .ok_or_else(|| {
            VexError::OfflineModeError(format!(
                "No cached archive found for {}@{}. Run 'vex install {}@{}' while online first.",
                tool.name(),
                version,
                tool.name(),
                version
            ))
        })?;

    let ctx = ui::UiContext::new();
    let progress = ui::Progress::new(
        &ctx,
        &format!(
            "Installing {} {} (offline)",
            tool.name().yellow(),
            version.yellow()
        ),
    );

    match tool.get_checksum(version, arch) {
        Ok(Some(expected)) => {
            progress.set_message("Verifying cached archive checksum");
            archive_cache.verify_checksum(&cached_archive, &expected)?;
        }
        _ => debug!("Skipping checksum verification in offline mode"),
    }

    let cache_dir = config::cache_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    let extract_dir = cache_dir.join(format!("{}-{}-extract-offline", tool.name(), version));

    let mut guard = CleanupGuard::new();
    guard.add(extract_dir.clone());

    progress.set_message("Extracting archive");
    fs::create_dir_all(&extract_dir)?;

    let tar_gz = fs::File::open(&cached_archive)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    extract_archive(&mut archive, &extract_dir)?;

    progress.set_message("Finalizing installation");
    let extracted_root = find_extracted_root(&extract_dir)?;

    let toolchains_dir = vex.join("toolchains").join(tool.name());
    fs::create_dir_all(&toolchains_dir)?;
    guard.add(final_dir.clone());
    fs::rename(&extracted_root, &final_dir)?;

    tool.post_install(&final_dir, arch)?;

    guard.disarm();
    let _ = fs::remove_dir_all(&extract_dir);

    progress.finish_with_success(&format!(
        "Installed {} {} (offline) to {}",
        tool.name().yellow(),
        version.yellow(),
        final_dir.display().to_string().dimmed()
    ));

    Ok(())
}
