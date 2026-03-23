/// Detect current shell from environment
///
/// # Returns
/// - `Some(String)` - Detected shell name (zsh, bash, fish, nu)
/// - `None` - Unable to detect shell
pub fn detect_shell() -> Option<String> {
    if let Ok(shell_path) = std::env::var("SHELL") {
        if let Some(shell_name) = shell_path.split('/').next_back() {
            match shell_name {
                "zsh" | "bash" | "fish" | "nu" => return Some(shell_name.to_string()),
                _ => {}
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let home_path = std::path::Path::new(&home);
        if home_path.join(".zshrc").exists() {
            return Some("zsh".to_string());
        }
        if home_path.join(".bashrc").exists() || home_path.join(".bash_profile").exists() {
            return Some("bash".to_string());
        }
        if home_path.join(".config/fish/config.fish").exists() {
            return Some("fish".to_string());
        }
        if home_path.join(".config/nushell/config.nu").exists() {
            return Some("nu".to_string());
        }
    }

    None
}

/// Get shell config file path
///
/// # Arguments
/// - `shell` - Shell type
///
/// # Returns
/// - `Ok(PathBuf)` - Config file path
/// - `Err(String)` - Unable to determine config path
pub fn get_shell_config_path(shell: &str) -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
    let home_path = std::path::Path::new(&home);

    match shell {
        "zsh" => Ok(home_path.join(".zshrc")),
        "bash" => {
            let bashrc = home_path.join(".bashrc");
            if bashrc.exists() {
                Ok(bashrc)
            } else {
                Ok(home_path.join(".bash_profile"))
            }
        }
        "fish" => Ok(home_path.join(".config/fish/config.fish")),
        "nu" | "nushell" => Ok(home_path.join(".config/nushell/config.nu")),
        _ => Err(format!("Unsupported shell: {}", shell)),
    }
}

/// Check if vex is already configured in shell config
///
/// # Arguments
/// - `config_path` - Path to shell config file
///
/// # Returns
/// - `Ok(bool)` - true if vex hook is already present
/// - `Err` - IO error
pub fn is_vex_configured(config_path: &std::path::Path) -> std::io::Result<bool> {
    if !config_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(config_path)?;
    Ok(content.contains("vex env") || content.contains("# vex shell integration"))
}
