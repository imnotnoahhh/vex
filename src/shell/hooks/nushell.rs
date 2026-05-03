use super::common::render_nushell_exports;
use crate::activation::ActivationPlan;

pub(super) fn generate_nushell_hook() -> String {
    r#"# vex shell integration
if ('VEX_ORIGINAL_PATH' not-in ($env | columns)) {
    $env.VEX_ORIGINAL_PATH = ($env.PATH | str join ':')
}
$env.PATH = ($env.PATH | prepend $"($env.HOME)/.vex/bin")
$env.PATH = ($env.PATH | prepend $"($env.HOME)/.vex/npm/prefix/bin")
$env.NPM_CONFIG_PREFIX = $"($env.HOME)/.vex/npm/prefix"
$env.NPM_CONFIG_USERCONFIG = $"($env.HOME)/.vex/npm/npmrc"
$env.CARGO_HOME = $"($env.HOME)/.vex/cargo"

def --env __vex_apply_exports [] {
    let exports_path = ($env.HOME | path join ".vex" "state" "env.nu")
    mkdir ($exports_path | path dirname)
    let status = (do -i { ^vex env nushell --exports } | complete)
    if $status.exit_code == 0 {
        $status.stdout | save -f $exports_path
        source $exports_path
    }
}

def --env __vex_use_if_found [] {
    do -i { ^vex use --auto } | complete | ignore
    __vex_apply_exports
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

pub(super) fn generate_nushell_exports(plan: &ActivationPlan) -> String {
    render_nushell_exports(plan)
}
