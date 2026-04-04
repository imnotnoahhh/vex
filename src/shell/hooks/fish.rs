use super::common::render_fish_exports;
use crate::activation::ActivationPlan;

pub(super) fn generate_fish_hook() -> String {
    r#"# vex shell integration
if not set -q VEX_ORIGINAL_PATH
    set -gx VEX_ORIGINAL_PATH $PATH
end
set -gx PATH $HOME/.vex/bin $PATH

function __vex_apply_exports
    set -l exports (vex env fish --exports 2>/dev/null)
    if test $status -eq 0
        eval $exports
    end
end

function __vex_use_if_found
    vex use --auto >/dev/null 2>/dev/null
    __vex_apply_exports
end

function __vex_on_pwd --on-variable PWD
    __vex_use_if_found
end

__vex_use_if_found
"#
    .to_string()
}

pub(super) fn generate_fish_exports(plan: &ActivationPlan) -> String {
    render_fish_exports(plan)
}
