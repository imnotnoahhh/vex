use super::common::{hook_prelude, render_bash_like_exports};
use crate::activation::ActivationPlan;

pub(super) fn generate_bash_hook() -> String {
    format!(
        r#"{}__vex_prompt_command() {{
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
        hook_prelude("bash")
    )
}

pub(super) fn generate_bash_exports(plan: &ActivationPlan) -> String {
    render_bash_like_exports(plan)
}
