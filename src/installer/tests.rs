use super::extract::{extract_archive, find_extracted_root};
use super::offline::install_offline;
use super::support::{check_disk_space, CleanupGuard};
use crate::archive_cache::ArchiveCache;
use crate::config;
use crate::error::{Result, VexError};
use crate::paths::vex_dir;
use crate::tools::{Arch, Tool, Version};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::io::empty;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tar::{Archive, Builder, EntryType, Header};
use tempfile::TempDir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct MockTool {
    fail_post_install: bool,
}

impl Tool for MockTool {
    fn name(&self) -> &str {
        "mock"
    }

    fn list_remote(&self) -> Result<Vec<Version>> {
        Ok(vec![])
    }

    fn download_url(&self, _version: &str, _arch: Arch) -> Result<String> {
        Ok("https://example.com/mock.tar.gz".to_string())
    }

    fn checksum_url(&self, _version: &str, _arch: Arch) -> Option<String> {
        None
    }

    fn bin_names(&self) -> Vec<&str> {
        vec!["mock"]
    }

    fn bin_subpath(&self) -> &str {
        "bin"
    }

    fn post_install(&self, _install_dir: &Path, _arch: Arch) -> Result<()> {
        if self.fail_post_install {
            Err(VexError::Parse("mock post-install failure".to_string()))
        } else {
            Ok(())
        }
    }
}

fn with_vex_home<T>(vex_home: &Path, f: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK.lock().unwrap();
    let original_vex_home = std::env::var("VEX_HOME").ok();

    std::env::set_var("VEX_HOME", vex_home);
    let result = f();

    if let Some(value) = original_vex_home {
        std::env::set_var("VEX_HOME", value);
    } else {
        std::env::remove_var("VEX_HOME");
    }

    result
}

fn append_directory(builder: &mut Builder<GzEncoder<fs::File>>, path: &str) {
    let mut header = Header::new_gnu();
    header.set_entry_type(EntryType::Directory);
    header.set_mode(0o755);
    header.set_size(0);
    header.set_cksum();
    builder.append_data(&mut header, path, empty()).unwrap();
}

fn append_file(builder: &mut Builder<GzEncoder<fs::File>>, path: &str, data: &[u8], mode: u32) {
    let mut header = Header::new_gnu();
    header.set_entry_type(EntryType::Regular);
    header.set_mode(mode);
    header.set_size(data.len() as u64);
    header.set_cksum();
    builder.append_data(&mut header, path, data).unwrap();
}

fn append_symlink(builder: &mut Builder<GzEncoder<fs::File>>, path: &str, target: &str, mode: u32) {
    let mut header = Header::new_gnu();
    header.set_entry_type(EntryType::Symlink);
    header.set_mode(mode);
    header.set_size(0);
    header.set_link_name(target).unwrap();
    header.set_cksum();
    builder.append_data(&mut header, path, empty()).unwrap();
}

fn write_mock_archive(path: &Path, root_dir: &str) {
    let file = fs::File::create(path).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);

    append_directory(&mut builder, &format!("{root_dir}/"));
    append_directory(&mut builder, &format!("{root_dir}/bin/"));
    append_file(
        &mut builder,
        &format!("{root_dir}/bin/mock"),
        b"#!/bin/sh\necho mock\n",
        0o755,
    );
    append_symlink(
        &mut builder,
        &format!("{root_dir}/bin/mock-link"),
        "mock",
        0o777,
    );

    builder.finish().unwrap();
}

fn write_archive_with_symlink(path: &Path, root_dir: &str, link_target: &str) {
    let file = fs::File::create(path).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);

    append_directory(&mut builder, &format!("{root_dir}/"));
    append_directory(&mut builder, &format!("{root_dir}/bin/"));
    append_symlink(
        &mut builder,
        &format!("{root_dir}/bin/mock-link"),
        link_target,
        0o777,
    );

    builder.finish().unwrap();
}

#[test]
fn test_check_disk_space_sufficient() {
    let temp_dir = TempDir::new().unwrap();
    let result = check_disk_space(temp_dir.path(), 1);
    assert!(result.is_ok());
}

#[test]
fn test_check_disk_space_insufficient() {
    let temp_dir = TempDir::new().unwrap();
    let result = check_disk_space(temp_dir.path(), 1024 * 1024 * 1024 * 1024 * 1024);
    assert!(result.is_err());
    if let Err(VexError::DiskSpace { need, available }) = result {
        assert!(need > available);
    } else {
        panic!("Expected DiskSpace error");
    }
}

#[test]
fn test_min_free_space_constant() {
    assert_eq!(config::MIN_FREE_SPACE_BYTES, 1536 * 1024 * 1024);
}

#[test]
fn test_cleanup_guard_new() {
    let guard = CleanupGuard::new();
    assert!(guard.paths.is_empty());
    assert!(!guard.disarmed);
}

#[test]
fn test_cleanup_guard_add() {
    let mut guard = CleanupGuard::new();
    guard.add(PathBuf::from("/tmp/test"));
    assert_eq!(guard.paths.len(), 1);
}

#[test]
fn test_cleanup_guard_disarm() {
    let mut guard = CleanupGuard::new();
    guard.add(PathBuf::from("/tmp/test"));
    guard.disarm();
    assert!(guard.disarmed);
}

#[test]
fn test_cleanup_guard_drop_disarmed() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test").unwrap();

    {
        let mut guard = CleanupGuard::new();
        guard.add(test_file.clone());
        guard.disarm();
    }

    assert!(test_file.exists());
}

#[test]
fn test_cleanup_guard_drop_armed_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test").unwrap();

    {
        let mut guard = CleanupGuard::new();
        guard.add(test_file.clone());
    }

    assert!(!test_file.exists());
}

#[test]
fn test_cleanup_guard_drop_armed_dir() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file.txt"), "test").unwrap();

    {
        let mut guard = CleanupGuard::new();
        guard.add(test_dir.clone());
    }

    assert!(!test_dir.exists());
}

#[test]
fn test_vex_dir_success() {
    let temp_dir = TempDir::new().unwrap();
    let vex_home = temp_dir.path().join(".vex");

    let result = with_vex_home(&vex_home, vex_dir);

    assert_eq!(result.unwrap(), vex_home);
}

#[test]
fn test_extract_archive_writes_files_and_finds_root() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("mock.tar.gz");
    let extract_dir = temp_dir.path().join("extract");
    write_mock_archive(&archive_path, "mock-1.0.0");

    fs::create_dir_all(&extract_dir).unwrap();
    let tar_gz = fs::File::open(&archive_path).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    extract_archive(&mut archive, &extract_dir).unwrap();
    let extracted_root = find_extracted_root(&extract_dir).unwrap();

    assert_eq!(extracted_root, extract_dir.join("mock-1.0.0"));
    assert!(extracted_root.join("bin/mock").is_file());
    assert_eq!(
        fs::read_to_string(extracted_root.join("bin/mock")).unwrap(),
        "#!/bin/sh\necho mock\n"
    );
    assert_eq!(
        fs::read_link(extracted_root.join("bin/mock-link")).unwrap(),
        PathBuf::from("mock")
    );
}

#[test]
fn test_extract_archive_rejects_absolute_symlink_target() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("unsafe-symlink.tar.gz");
    let extract_dir = temp_dir.path().join("extract");
    write_archive_with_symlink(&archive_path, "mock-1.0.0", "/etc/passwd");

    fs::create_dir_all(&extract_dir).unwrap();
    let tar_gz = fs::File::open(&archive_path).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    let error = extract_archive(&mut archive, &extract_dir).unwrap_err();
    assert!(
        matches!(error, VexError::Parse(message) if message.contains("absolute symlink target"))
    );
}

#[test]
fn test_install_offline_cleans_final_dir_when_post_install_fails() {
    let temp_dir = TempDir::new().unwrap();
    let vex_home = temp_dir.path().join(".vex");
    let source_archive = temp_dir.path().join("mock.tar.gz");
    let tool = MockTool {
        fail_post_install: true,
    };
    write_mock_archive(&source_archive, "mock-1.0.0");

    with_vex_home(&vex_home, || {
        let cache = ArchiveCache::new(&vex_home);
        cache
            .store_archive("mock", "1.0.0", "mock-1.0.0.tar.gz", &source_archive)
            .unwrap();

        let result = install_offline(&tool, "1.0.0");
        assert!(
            matches!(result, Err(VexError::Parse(message)) if message.contains("mock post-install failure"))
        );

        assert!(!vex_home.join("toolchains/mock/1.0.0").exists());
        assert!(!vex_home.join("cache/mock-1.0.0-extract-offline").exists());
        assert!(vex_home
            .join("cache/archives/mock/1.0.0/mock-1.0.0.tar.gz")
            .exists());
    });
}
