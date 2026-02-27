/// 生成 shell hook 脚本，用于 cd 时自动切换版本
pub fn generate_hook(shell: &str) -> Result<String, String> {
    match shell {
        "zsh" => Ok(generate_zsh_hook()),
        "bash" => Ok(generate_bash_hook()),
        _ => Err(format!(
            "Unsupported shell: {}. Supported: zsh, bash",
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
    fn test_unsupported_shell() {
        let result = generate_hook("fish");
        assert!(result.is_err());
    }
}
