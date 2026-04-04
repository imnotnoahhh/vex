//! Shell integration script generation module
//!
//! Generates shell hook scripts that automatically detect version files and switch tool versions on `cd`.
//! Supports zsh (chpwd), bash (PROMPT_COMMAND), fish (PWD variable monitoring), nushell (pre_prompt).

use crate::activation::ActivationPlan;

mod detection;
mod hooks;

pub use detection::{detect_shell, get_shell_config_path, is_vex_configured};
use hooks::{
    generate_bash_exports, generate_bash_hook, generate_fish_exports, generate_fish_hook,
    generate_nushell_exports, generate_nushell_hook, generate_zsh_exports, generate_zsh_hook,
};

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

pub fn generate_exports(shell: &str, plan: &ActivationPlan) -> Result<String, String> {
    match shell {
        "zsh" => Ok(generate_zsh_exports(plan)),
        "bash" => Ok(generate_bash_exports(plan)),
        "fish" => Ok(generate_fish_exports(plan)),
        "nu" | "nushell" => Ok(generate_nushell_exports(plan)),
        _ => Err(format!(
            "Unsupported shell: {}. Supported: zsh, bash, fish, nu",
            shell
        )),
    }
}

#[cfg(test)]
mod tests;
