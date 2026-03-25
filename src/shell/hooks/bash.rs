use super::common::vex_hook_function;

pub(super) fn generate_bash_hook() -> String {
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
