//! Shell integration script generation module
//!
//! Generates shell hook scripts that automatically detect version files and switch tool versions on `cd`.
//! Supports zsh (chpwd), bash (PROMPT_COMMAND), fish (PWD variable monitoring), nushell (pre_prompt).

/// Detect current shell from environment
///
/// # Returns
/// - `Some(String)` - Detected shell name (zsh, bash, fish, nu)
/// - `None` - Unable to detect shell
pub fn detect_shell() -> Option<String> {
    // Try SHELL environment variable first
    if let Ok(shell_path) = std::env::var("SHELL") {
        if let Some(shell_name) = shell_path.split('/').next_back() {
            match shell_name {
                "zsh" | "bash" | "fish" | "nu" => return Some(shell_name.to_string()),
                _ => {}
            }
        }
    }

    // Fallback: check common shell config files
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
            // Prefer .bashrc, fallback to .bash_profile
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

/// Generate hook script for specified shell
///
/// # Arguments
/// - `shell` - Shell type: `"zsh"`, `"bash"`, `"fish"`, `"nu"` / `"nushell"`
///
/// # Returns
/// - `Ok(String)` - Hook script content
/// - `Err(String)` - Unsupported shell type
pub fn generate_hook(shell: &str) -> Result<String, String> {
    match shell {
        "zsh" => Ok(generate_zsh_hook()),
        "bash" => Ok(generate_bash_hook()),
        "fish" => Ok(generate_fish_hook()),
        "nu" | "nushell" => Ok(generate_nushell_hook()),
        _ => Err(format!(
            "Unsupported shell: {}. Supported: zsh, bash, fish, nu",
            shell
        )),
    }
}

fn vex_hook_function() -> &'static str {
    r#"
__vex_use_if_found() {
    local dir="$PWD"
    local found=0

    # Search upward for version files
    while [ "$dir" != "" ]; do
        if [ -f "$dir/.tool-versions" ] || \
           [ -f "$dir/.node-version" ] || \
           [ -f "$dir/.go-version" ] || \
           [ -f "$dir/.java-version" ] || \
           [ -f "$dir/.rust-toolchain" ] || \
           [ -f "$dir/.python-version" ]; then
            vex use --auto 2>/dev/null
            found=1
            return
        fi
        dir="${dir%/*}"
    done

    # No project version found, fall back to global
    if [ $found -eq 0 ] && [ -f "$HOME/.vex/tool-versions" ]; then
        vex use --auto 2>/dev/null
    fi
}

__vex_activate_venv() {
    if [ -f "$PWD/.venv/bin/activate" ]; then
        if [ -z "$VIRTUAL_ENV" ] || [ "$VIRTUAL_ENV" != "$PWD/.venv" ]; then
            VIRTUAL_ENV_DISABLE_PROMPT=1 source "$PWD/.venv/bin/activate"
        fi
    elif [ -n "$VIRTUAL_ENV" ]; then
        deactivate 2>/dev/null || true
    fi
}
"#
}

fn generate_zsh_hook() -> String {
    format!(
        r#"# vex shell integration
export PATH="$HOME/.vex/bin:$PATH"
export CARGO_HOME="$HOME/.vex/cargo"
{}
autoload -U add-zsh-hook
add-zsh-hook chpwd __vex_use_if_found
add-zsh-hook chpwd __vex_activate_venv
__vex_use_if_found
__vex_activate_venv
"#,
        vex_hook_function()
    )
}

fn generate_bash_hook() -> String {
    format!(
        r#"# vex shell integration
export PATH="$HOME/.vex/bin:$PATH"
export CARGO_HOME="$HOME/.vex/cargo"
{}
__vex_prompt_command() {{
    if [ "$__VEX_PREV_DIR" != "$PWD" ]; then
        __VEX_PREV_DIR="$PWD"
        __vex_use_if_found
        __vex_activate_venv
    fi
}}

if [[ ";$PROMPT_COMMAND;" != *";__vex_prompt_command;"* ]]; then
    PROMPT_COMMAND="__vex_prompt_command;$PROMPT_COMMAND"
fi
__vex_use_if_found
__vex_activate_venv
"#,
        vex_hook_function()
    )
}

fn generate_fish_hook() -> String {
    r#"# vex shell integration
set -gx PATH $HOME/.vex/bin $PATH
set -gx CARGO_HOME $HOME/.vex/cargo

function __vex_use_if_found
    set -l dir $PWD
    set -l found 0

    # Search upward for version files
    while test "$dir" != ""
        if test -f "$dir/.tool-versions"; or \
           test -f "$dir/.node-version"; or \
           test -f "$dir/.go-version"; or \
           test -f "$dir/.java-version"; or \
           test -f "$dir/.rust-toolchain"; or \
           test -f "$dir/.python-version"
            vex use --auto 2>/dev/null
            set found 1
            return
        end
        set dir (string replace -r '/[^/]*$' '' "$dir")
    end

    # No project version found, fall back to global
    if test $found -eq 0; and test -f "$HOME/.vex/tool-versions"
        vex use --auto 2>/dev/null
    end
end

function __vex_activate_venv
    if test -f "$PWD/.venv/bin/activate.fish"
        if test -z "$VIRTUAL_ENV"; or test "$VIRTUAL_ENV" != "$PWD/.venv"
            source "$PWD/.venv/bin/activate.fish"
        end
    else if set -q VIRTUAL_ENV
        deactivate 2>/dev/null; or true
    end
end

function __vex_on_pwd --on-variable PWD
    __vex_use_if_found
    __vex_activate_venv
end

__vex_use_if_found
__vex_activate_venv
"#
    .to_string()
}

fn generate_nushell_hook() -> String {
    r#"# vex shell integration
$env.PATH = ($env.PATH | prepend $"($env.HOME)/.vex/bin")
$env.CARGO_HOME = $"($env.HOME)/.vex/cargo"

def --env __vex_use_if_found [] {
    mut dir = $env.PWD
    mut found = false

    # Search upward for version files
    while $dir != "" {
        if (
            ($dir | path join ".tool-versions" | path exists) or
            ($dir | path join ".node-version" | path exists) or
            ($dir | path join ".go-version" | path exists) or
            ($dir | path join ".java-version" | path exists) or
            ($dir | path join ".rust-toolchain" | path exists) or
            ($dir | path join ".python-version" | path exists)
        ) {
            vex use --auto | ignore
            $found = true
            return
        }
        $dir = ($dir | path dirname)
        if $dir == "/" {
            break
        }
    }

    # No project version found, fall back to global
    if (not $found) and (($env.HOME | path join ".vex" "tool-versions") | path exists) {
        vex use --auto | ignore
    }
}

def --env __vex_activate_venv [] {
    let venv_activate = ($env.PWD | path join ".venv" "bin" "activate.nu")
    if ($venv_activate | path exists) {
        if ($env | get -i VIRTUAL_ENV | is-empty) or ($env.VIRTUAL_ENV != ($env.PWD | path join ".venv")) {
            source $venv_activate
        }
    }
}

$env.config = ($env.config | upsert hooks {
    pre_prompt: ($env.config.hooks.pre_prompt | append {||
        __vex_use_if_found
        __vex_activate_venv
    })
})

__vex_use_if_found
__vex_activate_venv
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_zsh_hook() {
        let hook = generate_hook("zsh").unwrap();
        assert!(hook.contains("add-zsh-hook chpwd"));
        assert!(hook.contains("__vex_use_if_found"));
        assert!(hook.contains("__vex_activate_venv"));
        assert!(hook.contains(".tool-versions"));
        assert!(hook.contains("$HOME/.vex/bin"));
        assert!(hook.contains(".venv/bin/activate"));
    }

    #[test]
    fn test_generate_bash_hook() {
        let hook = generate_hook("bash").unwrap();
        assert!(hook.contains("PROMPT_COMMAND"));
        assert!(hook.contains("__vex_use_if_found"));
        assert!(hook.contains("__vex_activate_venv"));
        assert!(hook.contains(".tool-versions"));
        assert!(hook.contains(".venv/bin/activate"));
    }

    #[test]
    fn test_generate_fish_hook() {
        let hook = generate_hook("fish").unwrap();
        assert!(hook.contains("function __vex_use_if_found"));
        assert!(hook.contains("__vex_activate_venv"));
        assert!(hook.contains("on-variable PWD"));
        assert!(hook.contains(".tool-versions"));
        assert!(hook.contains("$HOME/.vex/bin"));
        assert!(hook.contains(".venv/bin/activate.fish"));
    }

    #[test]
    fn test_generate_nushell_hook() {
        let hook = generate_hook("nu").unwrap();
        assert!(hook.contains("def --env __vex_use_if_found"));
        assert!(hook.contains("__vex_activate_venv"));
        assert!(hook.contains("pre_prompt"));
        assert!(hook.contains(".tool-versions"));
        assert!(hook.contains("$env.PATH"));
    }

    #[test]
    fn test_generate_nushell_hook_alias() {
        let hook = generate_hook("nushell").unwrap();
        assert!(hook.contains("def --env __vex_use_if_found"));
    }

    #[test]
    fn test_unsupported_shell() {
        let result = generate_hook("powershell");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported shell"));
    }

    #[test]
    fn test_detect_shell() {
        // This test depends on the environment, so we just check it doesn't panic
        let _ = detect_shell();
    }

    #[test]
    fn test_get_shell_config_path() {
        // Test supported shells
        let zsh_path = get_shell_config_path("zsh");
        assert!(zsh_path.is_ok());
        assert!(zsh_path.unwrap().to_string_lossy().contains(".zshrc"));

        let bash_path = get_shell_config_path("bash");
        assert!(bash_path.is_ok());
        let path_str = bash_path.unwrap().to_string_lossy().to_string();
        assert!(path_str.contains(".bashrc") || path_str.contains(".bash_profile"));

        let fish_path = get_shell_config_path("fish");
        assert!(fish_path.is_ok());
        assert!(fish_path
            .unwrap()
            .to_string_lossy()
            .contains("config/fish/config.fish"));

        let nu_path = get_shell_config_path("nu");
        assert!(nu_path.is_ok());
        assert!(nu_path
            .unwrap()
            .to_string_lossy()
            .contains("config/nushell/config.nu"));

        // Test unsupported shell
        let result = get_shell_config_path("powershell");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_vex_configured() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Test with vex configured
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Some config").unwrap();
        writeln!(file, "eval \"$(vex env zsh)\"").unwrap();
        assert!(is_vex_configured(file.path()).unwrap());

        // Test without vex configured
        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(file2, "# Some other config").unwrap();
        assert!(!is_vex_configured(file2.path()).unwrap());

        // Test non-existent file
        let non_existent = std::path::Path::new("/tmp/non_existent_file_12345");
        assert!(!is_vex_configured(non_existent).unwrap());
    }

    #[test]
    fn test_generate_hook_contains_vex_bin() {
        for shell in &["zsh", "bash", "fish", "nu"] {
            let hook = generate_hook(shell).unwrap();
            assert!(
                hook.contains(".vex/bin") || hook.contains("$HOME/.vex/bin"),
                "Hook for {} should contain vex bin path",
                shell
            );
        }
    }

    #[test]
    fn test_generate_hook_contains_tool_versions() {
        for shell in &["zsh", "bash", "fish", "nu"] {
            let hook = generate_hook(shell).unwrap();
            assert!(
                hook.contains(".tool-versions"),
                "Hook for {} should check .tool-versions",
                shell
            );
        }
    }

    #[test]
    fn test_generate_hook_contains_venv_activation() {
        for shell in &["zsh", "bash", "fish", "nu"] {
            let hook = generate_hook(shell).unwrap();
            assert!(
                hook.contains("venv") || hook.contains("VIRTUAL_ENV"),
                "Hook for {} should handle Python venv",
                shell
            );
        }
    }

    #[test]
    fn test_get_shell_config_path_nushell_alias() {
        let nu_path = get_shell_config_path("nushell");
        assert!(nu_path.is_ok());
        assert!(nu_path
            .unwrap()
            .to_string_lossy()
            .contains("config/nushell/config.nu"));
    }

    #[test]
    fn test_is_vex_configured_with_different_formats() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Test with single quotes
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, "eval '$(vex env zsh)'").unwrap();
        assert!(is_vex_configured(file1.path()).unwrap());

        // Test with backticks
        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(file2, "eval `vex env bash`").unwrap();
        assert!(is_vex_configured(file2.path()).unwrap());

        // Test with spaces
        let mut file3 = NamedTempFile::new().unwrap();
        writeln!(file3, "  eval \"$(vex env zsh)\"  ").unwrap();
        assert!(is_vex_configured(file3.path()).unwrap());
    }

    #[test]
    fn test_generate_zsh_hook_structure() {
        let hook = generate_hook("zsh").unwrap();
        // Check for zsh-specific features
        assert!(hook.contains("add-zsh-hook"));
        assert!(hook.contains("chpwd"));
        assert!(hook.contains("__vex_use_if_found"));
    }

    #[test]
    fn test_generate_bash_hook_structure() {
        let hook = generate_hook("bash").unwrap();
        // Check for bash-specific features
        assert!(hook.contains("PROMPT_COMMAND"));
        assert!(hook.contains("__vex_prompt_command"));
    }

    #[test]
    fn test_generate_fish_hook_structure() {
        let hook = generate_hook("fish").unwrap();
        // Check for fish-specific features
        assert!(hook.contains("function"));
        assert!(hook.contains("on-variable PWD"));
        assert!(hook.contains("activate.fish"));
    }

    #[test]
    fn test_generate_nushell_hook_structure() {
        let hook = generate_hook("nu").unwrap();
        // Check for nushell-specific features
        assert!(hook.contains("def --env"));
        assert!(hook.contains("$env.config"));
        assert!(hook.contains("pre_prompt"));
    }

    #[test]
    fn test_unsupported_shells() {
        let unsupported = vec!["powershell", "cmd", "tcsh", "csh", "ksh"];
        for shell in unsupported {
            let result = generate_hook(shell);
            assert!(result.is_err(), "Shell {} should be unsupported", shell);
        }
    }
}
