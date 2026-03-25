use super::common::vex_hook_function;

pub(super) fn generate_zsh_hook() -> String {
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
