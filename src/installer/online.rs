use super::extract::{extract_archive, find_extracted_root};
use super::support::{check_disk_space, CleanupGuard};
use crate::archive_cache::ArchiveCache;
use crate::config;
use crate::downloader::{download_with_retry_in_current_context, verify_checksum};
use crate::error::{Result, VexError};
use crate::lock::InstallLock;
use crate::paths::vex_dir;
use crate::resolver;
use crate::tools::{Arch, Tool};
use crate::ui;
use flate2::read::GzDecoder;
use owo_colors::OwoColorize;
use std::fs;
use tar::Archive;
use tracing::{debug, info};

pub(super) fn install(tool: &dyn Tool, version: &str) -> Result<()> {
    info!("Starting installation: {}@{}", tool.name(), version);
    let arch = Arch::detect()?;
    debug!("Detected architecture: {:?}", arch);
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
        println!(
            "Use {} to switch to it.",
            format!("'vex use {}@{}'", tool.name(), version).cyan()
        );
        return Ok(());
    }

    let _lock = InstallLock::acquire(&vex, tool.name(), version)?;
    check_disk_space(&vex, config::MIN_FREE_SPACE_BYTES)?;

    let ctx = ui::UiContext::new();
    ui::info(&format!(
        "Installing {} {}",
        tool.name().yellow(),
        version.yellow()
    ));

    let cache_dir = config::cache_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    fs::create_dir_all(&cache_dir)?;
    let settings = config::load_effective_settings(&resolver::current_dir())?;

    let archive_name = format!("{}-{}.tar.gz", tool.name(), version);
    let archive_path = cache_dir.join(&archive_name);
    let extract_dir = cache_dir.join(format!("{}-{}-extract", tool.name(), version));

    let mut guard = CleanupGuard::new();
    guard.add(archive_path.clone());
    guard.add(extract_dir.clone());

    let download_url = config::rewrite_download_url_with_settings(
        &settings,
        tool.name(),
        &tool.download_url(version, arch)?,
    )?;
    download_with_retry_in_current_context(
        &download_url,
        &archive_path,
        settings.network.download_retries,
    )?;

    let progress = ui::Progress::new(&ctx, "Verifying checksum");

    let verified_checksum = match tool.get_checksum(version, arch) {
        Ok(Some(expected)) => {
            verify_checksum(&archive_path, &expected)?;
            Some(expected)
        }
        Ok(None) => None,
        Err(e) => {
            return Err(VexError::Parse(format!(
                "Failed to fetch checksum for verification: {}. Refusing to install unverified binary.",
                e
            )));
        }
    };

    let archive_cache = ArchiveCache::new(&vex);
    let _ = archive_cache.store_archive(tool.name(), version, &archive_name, &archive_path);

    progress.set_message("Extracting archive");
    fs::create_dir_all(&extract_dir)?;

    let tar_gz = fs::File::open(&archive_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    extract_archive(&mut archive, &extract_dir)?;

    progress.set_message("Finalizing installation");
    let extracted_dir = find_extracted_root(&extract_dir)?;

    let toolchains_dir = vex.join("toolchains").join(tool.name());
    fs::create_dir_all(&toolchains_dir)?;
    guard.add(final_dir.clone());
    fs::rename(&extracted_dir, &final_dir)?;

    tool.post_install(&final_dir, arch)?;

    if let Some(ref checksum) = verified_checksum {
        let checksum_file = final_dir.join(".vex-checksum");
        let _ = fs::write(&checksum_file, checksum);
    }

    guard.disarm();
    let _ = fs::remove_file(&archive_path);
    let _ = fs::remove_dir_all(&extract_dir);

    progress.finish_with_success(&format!(
        "Installed {} {} to {}",
        tool.name().yellow(),
        version.yellow(),
        final_dir.display().to_string().dimmed()
    ));

    if tool.name() == "node" {
        if let Ok(major_version) = version.split('.').next().unwrap_or("0").parse::<u32>() {
            if major_version >= 25 {
                println!();
                ui::info(&format!(
                    "Node.js 25+ no longer includes Corepack. To use pnpm or yarn, run: {}",
                    "corepack enable pnpm".cyan()
                ));
            }
        }
    }

    Ok(())
}
