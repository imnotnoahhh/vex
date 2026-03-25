use std::fs;
use std::path::Path;

pub fn path_size(path: &Path) -> u64 {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return 0;
    };
    let file_type = metadata.file_type();

    if file_type.is_symlink() {
        return 0;
    }

    if metadata.is_file() {
        return metadata.len();
    }

    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| path_size(&entry.path()))
        .sum()
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.2} GiB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.2} MiB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.2} KiB", bytes / KB)
    } else {
        format!("{} B", bytes as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn test_path_size_skips_symlinked_directories() {
        let dir = tempfile::tempdir().unwrap();
        let real_dir = dir.path().join("real");
        let linked_dir = dir.path().join("linked");
        fs::create_dir_all(&real_dir).unwrap();
        fs::write(real_dir.join("payload.bin"), vec![0_u8; 16]).unwrap();
        std::os::unix::fs::symlink(&real_dir, &linked_dir).unwrap();

        assert_eq!(path_size(dir.path()), 16);
    }
}
