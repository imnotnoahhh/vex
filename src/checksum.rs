use crate::config;
use crate::error::{Result, VexError};
use crate::tools::{Arch, Tool};
use sha2::Digest;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Policy that governs how vex reacts when a binary archive has no SHA-256 checksum.
///
/// `Strict` is the project default: vex refuses to install a binary that it cannot verify.
/// `AllowInsecure` is reserved for explicit, audited opt-ins (currently unused) and exists so
/// future code paths cannot regress to silent skipping by accident.
#[derive(Debug, Clone, Copy)]
pub(crate) enum VerificationPolicy {
    Strict,
    #[allow(dead_code)]
    AllowInsecure,
}

/// Verify a downloaded archive against the strongest checksum available.
///
/// Lookup order:
/// 1. A `pinned_checksum` (typically from `.tool-versions.lock` or `tool_metadata`).
/// 2. The tool's upstream checksum (`Tool::get_checksum`).
/// 3. Apply `policy`: `Strict` raises `ChecksumUnavailable`; `AllowInsecure` returns `Ok(None)`.
///
/// Returns the SHA-256 hex string that was verified, so callers can persist it for later audits.
pub(crate) fn verify_tool_archive(
    tool: &dyn Tool,
    version: &str,
    arch: Arch,
    archive_path: &Path,
    pinned_checksum: Option<&str>,
    policy: VerificationPolicy,
) -> Result<Option<String>> {
    if let Some(expected) = pinned_checksum {
        let expected = expected.trim();
        if expected.is_empty() {
            return checksum_missing(tool, version, policy);
        }
        verify_sha256(archive_path, expected)?;
        return Ok(Some(expected.to_string()));
    }

    match tool.get_checksum(version, arch) {
        Ok(Some(expected)) => {
            let expected = expected.trim();
            if expected.is_empty() {
                return checksum_missing(tool, version, policy);
            }
            verify_sha256(archive_path, expected)?;
            Ok(Some(expected.to_string()))
        }
        Ok(None) => checksum_missing(tool, version, policy),
        Err(err) => Err(VexError::Config(format!(
            "Failed to fetch checksum for {}@{}: {}. Refusing to install unverified binary.",
            tool.name(),
            version,
            err
        ))),
    }
}

fn checksum_missing(
    tool: &dyn Tool,
    version: &str,
    policy: VerificationPolicy,
) -> Result<Option<String>> {
    match policy {
        VerificationPolicy::Strict => Err(VexError::ChecksumUnavailable {
            tool: tool.name().to_string(),
            version: version.to_string(),
        }),
        VerificationPolicy::AllowInsecure => Ok(None),
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn sha256_hex(file_path: &Path) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = vec![0u8; config::CHECKSUM_BUFFER_SIZE];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(encode_hex(hasher.finalize().as_ref()))
}

pub(crate) fn verify_sha256(file_path: &Path, expected: &str) -> Result<()> {
    let actual = sha256_hex(file_path)?;
    if actual == expected {
        Ok(())
    } else {
        Err(VexError::ChecksumMismatch {
            expected: expected.to_string(),
            actual,
        })
    }
}
