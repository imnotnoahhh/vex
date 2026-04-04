use super::common::{hook_prelude, render_bash_like_exports};
use crate::activation::ActivationPlan;

pub(super) fn generate_zsh_hook() -> String {
    format!(
        r#"{}autoload -U add-zsh-hook
add-zsh-hook chpwd __vex_use_if_found
__vex_use_if_found
"#,
        hook_prelude("zsh")
    )
}

pub(super) fn generate_zsh_exports(plan: &ActivationPlan) -> String {
    render_bash_like_exports(plan)
}
