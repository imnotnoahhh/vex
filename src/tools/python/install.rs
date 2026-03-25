use crate::error::Result;
use std::fs;
use std::path::Path;

pub(super) fn rewire_placeholder_binaries(install_dir: &Path) -> Result<()> {
    let bin_dir = install_dir.join("bin");
    let Some(versioned_python) = find_versioned_python_binary(&bin_dir)? else {
        return Ok(());
    };

    for (placeholder, target) in replacement_targets(&versioned_python) {
        let placeholder_path = bin_dir.join(&placeholder);
        let target_path = bin_dir.join(&target);

        if should_replace_placeholder(&placeholder_path, &target_path)? {
            fs::remove_file(&placeholder_path)?;
            std::os::unix::fs::symlink(&target, &placeholder_path)?;
        }
    }

    Ok(())
}

fn find_versioned_python_binary(bin_dir: &Path) -> Result<Option<String>> {
    let versioned = fs::read_dir(bin_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .find(|name| {
            name.starts_with("python3.")
                && name
                    .chars()
                    .last()
                    .map(|ch| ch.is_ascii_digit())
                    .unwrap_or(false)
        });

    Ok(versioned)
}

fn replacement_targets(versioned_python: &str) -> Vec<(String, String)> {
    let minor = versioned_python.trim_start_matches("python");
    vec![
        ("python3".to_string(), versioned_python.to_string()),
        ("python".to_string(), versioned_python.to_string()),
        ("2to3".to_string(), format!("2to3-{}", minor)),
        ("idle3".to_string(), format!("idle{}", minor)),
        ("pydoc3".to_string(), format!("pydoc{}", minor)),
        (
            "python3-config".to_string(),
            format!("python{}-config", minor),
        ),
    ]
}

fn should_replace_placeholder(placeholder_path: &Path, target_path: &Path) -> Result<bool> {
    Ok(placeholder_path.exists()
        && target_path.exists()
        && fs::metadata(placeholder_path)?.len() == 0
        && fs::metadata(target_path)?.len() > 0)
}
