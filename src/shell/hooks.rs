mod bash;
mod common;
mod fish;
mod nushell;
mod zsh;

pub(super) fn generate_bash_hook() -> String {
    bash::generate_bash_hook()
}

pub(super) fn generate_fish_hook() -> String {
    fish::generate_fish_hook()
}

pub(super) fn generate_nushell_hook() -> String {
    nushell::generate_nushell_hook()
}

pub(super) fn generate_zsh_hook() -> String {
    zsh::generate_zsh_hook()
}
