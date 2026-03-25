use crate::error::{Result, VexError};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(super) fn extract_binary_from_tarball(tarball: &Path, current_exe: &Path) -> Result<PathBuf> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let file = fs::File::open(tarball)?;
    let gz = GzDecoder::new(file);
    unpack_vex_from_archive(Archive::new(gz), current_exe)
}

pub(super) fn extract_binary_from_tarball_xz(
    tarball: &Path,
    current_exe: &Path,
) -> Result<PathBuf> {
    use tar::Archive;
    use xz2::read::XzDecoder;

    let file = fs::File::open(tarball)?;
    let xz = XzDecoder::new(file);
    unpack_vex_from_archive(Archive::new(xz), current_exe)
}

fn unpack_vex_from_archive<R: Read>(
    mut archive: tar::Archive<R>,
    current_exe: &Path,
) -> Result<PathBuf> {
    let out_path = current_exe.with_extension("extracted_tmp");

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if file_name == "vex" {
            entry.unpack(&out_path)?;
            return Ok(out_path);
        }
    }

    Err(VexError::Parse(
        "Could not find 'vex' binary inside the release archive".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    #[test]
    fn test_extract_binary_from_tarball_found() {
        let dir = TempDir::new().unwrap();
        let tarball = dir.path().join("release.tar.gz");
        let fake_exe = dir.path().join("fake_vex");

        let gz = fs::File::create(&tarball).unwrap();
        let enc = GzEncoder::new(gz, Compression::default());
        let mut ar = tar::Builder::new(enc);
        let content = b"#!/bin/sh\necho vex";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        ar.append_data(
            &mut header,
            "vex-0.1.7-aarch64-apple-darwin/vex",
            &content[..],
        )
        .unwrap();
        ar.into_inner().unwrap().finish().unwrap();

        let result = extract_binary_from_tarball(&tarball, &fake_exe);
        assert!(result.is_ok(), "extract failed: {:?}", result.err());
        assert!(result.unwrap().exists());
    }

    #[test]
    fn test_extract_binary_from_tarball_not_found() {
        let dir = TempDir::new().unwrap();
        let tarball = dir.path().join("empty.tar.gz");
        let fake_exe = dir.path().join("fake_vex");

        let gz = fs::File::create(&tarball).unwrap();
        let enc = GzEncoder::new(gz, Compression::default());
        let mut ar = tar::Builder::new(enc);
        ar.finish().unwrap();

        let result = extract_binary_from_tarball(&tarball, &fake_exe);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_binary_permissions() {
        let dir = TempDir::new().unwrap();
        let tarball = dir.path().join("release.tar.gz");
        let fake_exe = dir.path().join("fake_vex");

        let gz = fs::File::create(&tarball).unwrap();
        let enc = GzEncoder::new(gz, Compression::default());
        let mut ar = tar::Builder::new(enc);
        let content = b"#!/bin/sh\necho vex";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        ar.append_data(
            &mut header,
            "vex-0.1.7-aarch64-apple-darwin/vex",
            &content[..],
        )
        .unwrap();
        ar.into_inner().unwrap().finish().unwrap();

        let result = extract_binary_from_tarball(&tarball, &fake_exe);
        assert!(result.is_ok());

        let out = result.unwrap();
        let metadata = fs::metadata(&out).unwrap();
        assert!(metadata.permissions().mode() & 0o111 != 0);
    }
}
