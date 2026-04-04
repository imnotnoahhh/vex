mod bash;
mod common;
mod fish;
mod nushell;
mod zsh;

use crate::activation::ActivationPlan;

pub(super) fn generate_bash_hook() -> String {
    bash::generate_bash_hook()
}

pub(super) fn generate_bash_exports(plan: &ActivationPlan) -> String {
    bash::generate_bash_exports(plan)
}

pub(super) fn generate_fish_hook() -> String {
    fish::generate_fish_hook()
}

pub(super) fn generate_fish_exports(plan: &ActivationPlan) -> String {
    fish::generate_fish_exports(plan)
}

pub(super) fn generate_nushell_hook() -> String {
    nushell::generate_nushell_hook()
}

pub(super) fn generate_nushell_exports(plan: &ActivationPlan) -> String {
    nushell::generate_nushell_exports(plan)
}

pub(super) fn generate_zsh_hook() -> String {
    zsh::generate_zsh_hook()
}

pub(super) fn generate_zsh_exports(plan: &ActivationPlan) -> String {
    zsh::generate_zsh_exports(plan)
}
