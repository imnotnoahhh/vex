use crate::checksum;
use crate::cli::rust::{RustCommands, RustExtensionArgs, RustExtensionCommand};
use crate::config;
use crate::downloader::download_with_retry_in_current_context;
use crate::error::{Result, VexError};
use crate::requested_versions;
use crate::resolver;
use crate::tool_metadata::{
    self, ExtensionMetadata, OwnershipMetadata, ProvenanceMetadata, ToolchainMetadata,
};
use crate::tools;
use crate::tools::rust::dist::target_triple;
use crate::tools::rust::install::{
    link_preview_lib_component, link_rust_src_component, link_standard_library_component,
    remove_component_link,
};
use crate::tools::rust::manifest::{self, ChannelManifest};
use crate::version_state;
use chrono::Utc;
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;

pub fn run(args: &crate::cli::rust::RustArgs) -> Result<()> {
    match &args.command {
        RustCommands::Target(extension) => run_extension("target", extension),
        RustCommands::Component(extension) => run_extension("component", extension),
    }
}

fn run_extension(kind: &str, args: &RustExtensionArgs) -> Result<()> {
    let arch = tools::Arch::detect()?;
    let host_target = target_triple(arch);
    let (version, install_dir) = active_rust_toolchain()?;
    let manifest = manifest::fetch_channel_manifest(&version)
        .or_else(|_| manifest::fetch_channel_manifest("stable"))?;

    match &args.command {
        RustExtensionCommand::List => list_extensions(kind, host_target, &manifest, &install_dir),
        RustExtensionCommand::Add { names } => {
            for name in names {
                install_extension(
                    kind,
                    name,
                    arch,
                    host_target,
                    &version,
                    &manifest,
                    &install_dir,
                )?;
            }
            Ok(())
        }
        RustExtensionCommand::Remove { names } => {
            for name in names {
                remove_extension(kind, name, &install_dir)?;
            }
            Ok(())
        }
    }
}

fn list_extensions(
    kind: &str,
    host_target: &str,
    manifest: &ChannelManifest,
    install_dir: &Path,
) -> Result<()> {
    let metadata = tool_metadata::read_metadata(install_dir)?;
    let managed = metadata
        .as_ref()
        .map(|metadata| {
            metadata
                .extensions
                .iter()
                .filter(|extension| extension.kind == kind)
                .map(|extension| extension.name.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();

    let (installed, available) = if kind == "target" {
        (
            installed_targets(install_dir),
            manifest.available_targets().into_iter().collect::<Vec<_>>(),
        )
    } else {
        (
            installed_components(install_dir),
            manifest.available_components(host_target),
        )
    };

    println!("Active Rust toolchain: {}", install_dir.display());
    println!();
    println!("Installed {}s:", kind);
    if installed.is_empty() {
        println!("  (none)");
    } else {
        for item in &installed {
            if managed.contains(item) {
                println!("  {} (managed)", item);
            } else {
                println!("  {}", item);
            }
        }
    }
    println!();
    println!("Available official {}s:", kind);
    for item in available {
        println!("  {}", item);
    }
    Ok(())
}

fn install_extension(
    kind: &str,
    name: &str,
    arch: tools::Arch,
    host_target: &str,
    version: &str,
    manifest: &ChannelManifest,
    install_dir: &Path,
) -> Result<()> {
    let component_dir = install_dir.join(component_dir_name(kind, name));
    if component_dir.exists() {
        println!("rust {} {} is already present", kind, name);
        return Ok(());
    }

    let artifact = if kind == "target" {
        manifest.artifact_for_target(name)?
    } else {
        manifest.artifact_for_component(name, host_target)?
    };

    let settings = config::load_effective_settings_for_current_dir()?;
    let temp_dir = tempfile::tempdir()?;
    let archive_name = artifact
        .url
        .split('/')
        .next_back()
        .unwrap_or("rust-extension.tar.xz");
    let archive_path = temp_dir.path().join(archive_name);
    download_with_retry_in_current_context(
        &artifact.url,
        &archive_path,
        settings.network.download_retries,
    )?;
    checksum::verify_sha256(&archive_path, &artifact.checksum)?;

    let extract_dir = temp_dir.path().join("extract");
    fs::create_dir_all(&extract_dir)?;
    unpack_archive(&archive_path, &extract_dir)?;
    let extracted_root = first_subdir(&extract_dir)?;
    let extracted_component = extracted_root.join(component_dir_name(kind, name));
    if !extracted_component.exists() {
        return Err(VexError::Parse(format!(
            "Downloaded Rust {} archive did not contain {}",
            kind,
            component_dir_name(kind, name)
        )));
    }

    fs::rename(&extracted_component, &component_dir)?;
    let owned_paths = link_extension(kind, name, arch, install_dir)?;
    update_metadata_after_add(
        kind,
        name,
        version,
        install_dir,
        &artifact.url,
        &artifact.checksum,
        owned_paths,
    )?;
    println!("Installed rust {} {}", kind, name);
    Ok(())
}

fn remove_extension(kind: &str, name: &str, install_dir: &Path) -> Result<()> {
    let mut metadata = load_or_bootstrap_metadata(install_dir)?;
    let Some(index) = metadata
        .extensions
        .iter()
        .position(|extension| extension.kind == kind && extension.name == name)
    else {
        println!("rust {} {} is not managed by vex", kind, name);
        return Ok(());
    };

    let extension = metadata.extensions.remove(index);
    for path in extension.owned_paths.iter().rev() {
        remove_owned_path(Path::new(path))?;
    }

    tool_metadata::write_metadata(install_dir, &metadata)?;
    println!("Removed rust {} {}", kind, name);
    Ok(())
}

fn active_rust_toolchain() -> Result<(String, PathBuf)> {
    let cwd = resolver::current_dir();
    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let toolchains_dir = config::toolchains_dir().ok_or(VexError::HomeDirectoryNotFound)?;
    let requested = resolver::resolve_versions(&cwd);

    if let Some(version) = requested.get("rust") {
        let resolved = requested_versions::resolve_installed_version(&vex_dir, "rust", version)?
            .ok_or_else(|| VexError::VersionNotFound {
                tool: "rust".to_string(),
                version: version.clone(),
                suggestions: String::new(),
            })?;
        return Ok((resolved.clone(), toolchains_dir.join("rust").join(resolved)));
    }

    let current_versions = version_state::read_current_versions(&vex_dir)?;
    let version = current_versions
        .get("rust")
        .cloned()
        .ok_or_else(|| VexError::Config("No active Rust toolchain was found.".to_string()))?;
    Ok((version.clone(), toolchains_dir.join("rust").join(version)))
}

fn installed_targets(install_dir: &Path) -> Vec<String> {
    let mut targets = fs::read_dir(install_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(|entry| entry.ok()))
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            name.strip_prefix("rust-std-")
                .map(|target| target.to_string())
        })
        .collect::<Vec<_>>();
    targets.sort();
    targets
}

fn installed_components(install_dir: &Path) -> Vec<String> {
    let mut components = fs::read_dir(install_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(|entry| entry.ok()))
        .filter(|entry| {
            entry
                .file_type()
                .ok()
                .map(|kind| kind.is_dir())
                .unwrap_or(false)
        })
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| !name.starts_with("rust-std-") && !name.starts_with('.'))
        .collect::<Vec<_>>();
    components.sort();
    components
}

fn component_dir_name(kind: &str, name: &str) -> String {
    if kind == "target" {
        format!("rust-std-{}", name)
    } else {
        name.to_string()
    }
}

fn unpack_archive(archive_path: &Path, extract_dir: &Path) -> Result<()> {
    let file = fs::File::open(archive_path)?;
    if archive_path.extension().and_then(|ext| ext.to_str()) == Some("xz") {
        let decoder = xz2::read::XzDecoder::new(file);
        let mut archive = Archive::new(Box::new(decoder) as Box<dyn Read>);
        archive.unpack(extract_dir)?;
    } else {
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = Archive::new(Box::new(decoder) as Box<dyn Read>);
        archive.unpack(extract_dir)?;
    }
    Ok(())
}

fn first_subdir(root: &Path) -> Result<PathBuf> {
    fs::read_dir(root)?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            entry
                .file_type()
                .ok()
                .map(|kind| kind.is_dir())
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .ok_or_else(|| VexError::Parse("No extracted Rust component directory found".to_string()))
}

fn link_extension(
    kind: &str,
    name: &str,
    _arch: tools::Arch,
    install_dir: &Path,
) -> Result<Vec<String>> {
    let mut owned_paths = vec![install_dir
        .join(component_dir_name(kind, name))
        .display()
        .to_string()];

    match kind {
        "target" => {
            link_standard_library_component(install_dir, name)?;
            let lib = install_dir.join("rustc/lib/rustlib").join(name).join("lib");
            if lib.exists() {
                owned_paths.push(lib.display().to_string());
            }
        }
        "component" if name == "rust-src" => {
            link_rust_src_component(install_dir)?;
            let src = install_dir.join("rustc/lib/rustlib/src");
            if src.exists() {
                owned_paths.push(src.display().to_string());
            }
        }
        "component"
            if matches!(
                name,
                "clippy-preview" | "rustfmt-preview" | "rust-analyzer-preview"
            ) =>
        {
            link_preview_lib_component(install_dir, name)?;
            let lib = install_dir.join(name).join("lib");
            if lib.exists() {
                owned_paths.push(lib.display().to_string());
            }
        }
        _ => {}
    }

    Ok(owned_paths)
}

fn update_metadata_after_add(
    kind: &str,
    name: &str,
    version: &str,
    install_dir: &Path,
    source_url: &str,
    checksum: &str,
    owned_paths: Vec<String>,
) -> Result<()> {
    let mut metadata = load_or_bootstrap_metadata(install_dir)?;
    metadata
        .extensions
        .retain(|extension| !(extension.kind == kind && extension.name == name));
    metadata.extensions.push(ExtensionMetadata {
        kind: kind.to_string(),
        name: name.to_string(),
        source_url: Some(source_url.to_string()),
        checksum: Some(checksum.to_string()),
        installed_at: Utc::now().to_rfc3339(),
        owned_paths,
    });
    metadata.version = version.to_string();
    tool_metadata::write_metadata(install_dir, &metadata)
}

fn load_or_bootstrap_metadata(install_dir: &Path) -> Result<ToolchainMetadata> {
    if let Some(metadata) = tool_metadata::read_metadata(install_dir)? {
        return Ok(metadata);
    }

    let vex_dir = config::vex_home().ok_or(VexError::HomeDirectoryNotFound)?;
    let version = install_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| VexError::Parse("Invalid Rust toolchain path".to_string()))?;
    let tool = tools::get_tool("rust")?;
    let managed = tool.managed_environment(&vex_dir, Some(install_dir));

    Ok(ToolchainMetadata {
        tool: "rust".to_string(),
        version: version.to_string(),
        provenance: ProvenanceMetadata {
            source_url: Some(manifest::manifest_url(version)),
            mirror_url: None,
            checksum: None,
            installed_at: Utc::now().to_rfc3339(),
        },
        ownership: OwnershipMetadata {
            vex_owned: vec![install_dir.display().to_string()],
            managed_user_state: managed.owned_home_dirs,
            project_owned: managed.project_owned_dirs,
        },
        extensions: Vec::new(),
        managed_env: managed.managed_env,
    })
}

fn remove_owned_path(path: &Path) -> Result<()> {
    if path.symlink_metadata().is_err() {
        return Ok(());
    }

    if path
        .symlink_metadata()
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
    {
        remove_component_link(path)?;
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}
