//! Shell integration script generation module
//!
//! Generates shell hook scripts that automatically detect version files and switch tool versions on `cd`.
//! Supports zsh (chpwd), bash (PROMPT_COMMAND), fish (PWD variable monitoring), nushell (pre_prompt).

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
    while [ "$dir" != "" ]; do
        if [ -f "$dir/.tool-versions" ] || \
           [ -f "$dir/.node-version" ] || \
           [ -f "$dir/.go-version" ] || \
           [ -f "$dir/.java-version" ] || \
           [ -f "$dir/.rust-toolchain" ]; then
            vex use --auto 2>/dev/null
            return
        fi
        dir="${dir%/*}"
    done
}
"#
}

fn generate_zsh_hook() -> String {
    format!(
        r#"# vex shell integration
export PATH="$HOME/.vex/bin:$PATH"
{}
autoload -U add-zsh-hook
add-zsh-hook chpwd __vex_use_if_found
__vex_use_if_found
"#,
        vex_hook_function()
    )
}

fn generate_bash_hook() -> String {
    format!(
        r#"# vex shell integration
export PATH="$HOME/.vex/bin:$PATH"
{}
__vex_prompt_command() {{
    if [ "$__VEX_PREV_DIR" != "$PWD" ]; then
        __VEX_PREV_DIR="$PWD"
        __vex_use_if_found
    fi
}}

if [[ ";$PROMPT_COMMAND;" != *";__vex_prompt_command;"* ]]; then
    PROMPT_COMMAND="__vex_prompt_command;$PROMPT_COMMAND"
fi
__vex_use_if_found
"#,
        vex_hook_function()
    )
}

fn generate_fish_hook() -> String {
    r#"# vex shell integration
set -gx PATH $HOME/.vex/bin $PATH

function __vex_use_if_found
    set -l dir $PWD
    while test "$dir" != ""
        if test -f "$dir/.tool-versions"; or \
           test -f "$dir/.node-version"; or \
           test -f "$dir/.go-version"; or \
           test -f "$dir/.java-version"; or \
           test -f "$dir/.rust-toolchain"
            vex use --auto 2>/dev/null
            return
        end
        set dir (string replace -r '/[^/]*$' '' "$dir")
    end
end

function __vex_on_pwd --on-variable PWD
    __vex_use_if_found
end

__vex_use_if_found
"#
    .to_string()
}

fn generate_nushell_hook() -> String {
    r#"# vex shell integration
$env.PATH = ($env.PATH | prepend $"($env.HOME)/.vex/bin")

def --env __vex_use_if_found [] {
    mut dir = $env.PWD
    while $dir != "" {
        if (
            ($dir | path join ".tool-versions" | path exists) or
            ($dir | path join ".node-version" | path exists) or
            ($dir | path join ".go-version" | path exists) or
            ($dir | path join ".java-version" | path exists) or
            ($dir | path join ".rust-toolchain" | path exists)
        ) {
            vex use --auto | ignore
            return
        }
        $dir = ($dir | path dirname)
        if $dir == "/" {
            break
        }
    }
}

$env.config = ($env.config | upsert hooks {
    pre_prompt: ($env.config.hooks.pre_prompt | append {||
        __vex_use_if_found
    })
})

__vex_use_if_found
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
        assert!(hook.contains(".tool-versions"));
        assert!(hook.contains("$HOME/.vex/bin"));
    }

    #[test]
    fn test_generate_bash_hook() {
        let hook = generate_hook("bash").unwrap();
        assert!(hook.contains("PROMPT_COMMAND"));
        assert!(hook.contains("__vex_use_if_found"));
        assert!(hook.contains(".tool-versions"));
    }

    #[test]
    fn test_generate_fish_hook() {
        let hook = generate_hook("fish").unwrap();
        assert!(hook.contains("function __vex_use_if_found"));
        assert!(hook.contains("on-variable PWD"));
        assert!(hook.contains(".tool-versions"));
        assert!(hook.contains("$HOME/.vex/bin"));
    }

    #[test]
    fn test_generate_nushell_hook() {
        let hook = generate_hook("nu").unwrap();
        assert!(hook.contains("def --env __vex_use_if_found"));
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
}
