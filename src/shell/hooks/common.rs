use crate::activation::{self, ActivationPlan};

pub(super) fn hook_prelude(shell: &str) -> String {
    format!(
        r#"# vex shell integration
if [ -z "${{VEX_ORIGINAL_PATH+x}}" ]; then
    export VEX_ORIGINAL_PATH="$PATH"
fi
export PATH="$HOME/.vex/bin:$PATH"

__vex_apply_exports() {{
    local exports
    exports="$(vex env {shell} --exports 2>/dev/null)" || return 0
    eval "$exports"
}}

__vex_use_if_found() {{
    vex use --auto >/dev/null 2>&1 || true
    __vex_apply_exports
}}
"#
    )
}

pub(super) fn bash_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', r#"'\''"#))
}

pub(super) fn fish_escape(value: &str) -> String {
    format!("'{}'", value.replace('\\', r#"\\"#).replace('\'', r#"\'"#))
}

pub(super) fn nushell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(super) fn render_bash_like_exports(plan: &ActivationPlan) -> String {
    let mut lines = Vec::new();
    for key in &plan.unset_env {
        lines.push(format!("unset {}", key));
    }
    for (key, value) in &plan.set_env {
        lines.push(format!("export {}={}", key, bash_escape(value)));
    }
    if let Ok(path) = activation::shell_path(plan) {
        lines.push(format!("export PATH={}", bash_escape(&path)));
    }
    lines.join("\n") + "\n"
}

pub(super) fn render_fish_exports(plan: &ActivationPlan) -> String {
    let mut lines = Vec::new();
    for key in &plan.unset_env {
        lines.push(format!("set -e {}", key));
    }
    for (key, value) in &plan.set_env {
        lines.push(format!("set -gx {} {}", key, fish_escape(value)));
    }
    if let Ok(path) = activation::shell_path(plan) {
        let segments = path
            .split(':')
            .filter(|segment| !segment.is_empty())
            .map(fish_escape)
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(format!("set -gx PATH {}", segments));
    }
    lines.join("\n") + "\n"
}

pub(super) fn render_nushell_exports(plan: &ActivationPlan) -> String {
    let mut lines = Vec::new();
    for key in &plan.unset_env {
        lines.push(format!("hide-env {}", key));
    }
    for (key, value) in &plan.set_env {
        lines.push(format!("$env.{} = {}", key, nushell_escape(value)));
    }
    if let Ok(path) = activation::shell_path(plan) {
        lines.push(format!(
            "$env.PATH = ({} | split row ':')",
            nushell_escape(&path)
        ));
    }
    lines.join("\n") + "\n"
}
