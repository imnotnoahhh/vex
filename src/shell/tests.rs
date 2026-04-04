use super::*;

#[test]
fn test_generate_zsh_hook() {
    let hook = generate_hook("zsh").unwrap();
    assert!(hook.contains("add-zsh-hook chpwd"));
    assert!(hook.contains("__vex_use_if_found"));
    assert!(hook.contains("__vex_apply_exports"));
    assert!(hook.contains("vex env zsh --exports"));
    assert!(hook.contains("$HOME/.vex/bin"));
    assert!(hook.contains("VEX_ORIGINAL_PATH"));
}

#[test]
fn test_generate_bash_hook() {
    let hook = generate_hook("bash").unwrap();
    assert!(hook.contains("PROMPT_COMMAND"));
    assert!(hook.contains("__vex_use_if_found"));
    assert!(hook.contains("__vex_apply_exports"));
    assert!(hook.contains("vex env bash --exports"));
    assert!(hook.contains("VEX_ORIGINAL_PATH"));
}

#[test]
fn test_generate_fish_hook() {
    let hook = generate_hook("fish").unwrap();
    assert!(hook.contains("function __vex_use_if_found"));
    assert!(hook.contains("__vex_apply_exports"));
    assert!(hook.contains("on-variable PWD"));
    assert!(hook.contains("vex env fish --exports"));
    assert!(hook.contains("$HOME/.vex/bin"));
    assert!(hook.contains("VEX_ORIGINAL_PATH"));
}

#[test]
fn test_generate_nushell_hook() {
    let hook = generate_hook("nu").unwrap();
    assert!(hook.contains("def --env __vex_use_if_found"));
    assert!(hook.contains("__vex_apply_exports"));
    assert!(hook.contains("pre_prompt"));
    assert!(hook.contains("vex env nushell --exports"));
    assert!(hook.contains("$env.PATH"));
}

#[test]
fn test_generate_nushell_hook_alias() {
    let hook = generate_hook("nushell").unwrap();
    assert!(hook.contains("def --env __vex_use_if_found"));
}

#[test]
fn test_unsupported_shell() {
    let result = generate_hook("powershell");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported shell"));
}

#[test]
fn test_detect_shell() {
    let _ = detect_shell();
}

#[test]
fn test_get_shell_config_path() {
    let zsh_path = get_shell_config_path("zsh");
    assert!(zsh_path.is_ok());
    assert!(zsh_path.unwrap().to_string_lossy().contains(".zshrc"));

    let bash_path = get_shell_config_path("bash");
    assert!(bash_path.is_ok());
    let path_str = bash_path.unwrap().to_string_lossy().to_string();
    assert!(path_str.contains(".bashrc") || path_str.contains(".bash_profile"));

    let fish_path = get_shell_config_path("fish");
    assert!(fish_path.is_ok());
    assert!(fish_path
        .unwrap()
        .to_string_lossy()
        .contains("config/fish/config.fish"));

    let nu_path = get_shell_config_path("nu");
    assert!(nu_path.is_ok());
    assert!(nu_path
        .unwrap()
        .to_string_lossy()
        .contains("config/nushell/config.nu"));

    let result = get_shell_config_path("powershell");
    assert!(result.is_err());
}

#[test]
fn test_is_vex_configured() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# Some config").unwrap();
    writeln!(file, "eval \"$(vex env zsh)\"").unwrap();
    assert!(is_vex_configured(file.path()).unwrap());

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "# Some other config").unwrap();
    assert!(!is_vex_configured(file2.path()).unwrap());

    let non_existent = std::path::Path::new("/tmp/non_existent_file_12345");
    assert!(!is_vex_configured(non_existent).unwrap());
}

#[test]
fn test_generate_hook_contains_vex_bin() {
    for shell in &["zsh", "bash", "fish", "nu"] {
        let hook = generate_hook(shell).unwrap();
        assert!(
            hook.contains(".vex/bin") || hook.contains("$HOME/.vex/bin"),
            "Hook for {} should contain vex bin path",
            shell
        );
    }
}

#[test]
fn test_generate_hook_contains_tool_versions() {
    for shell in &["zsh", "bash", "fish", "nu"] {
        let hook = generate_hook(shell).unwrap();
        assert!(
            hook.contains("vex use --auto"),
            "Hook for {} should auto-switch on directory changes",
            shell
        );
    }
}

#[test]
fn test_generate_hook_contains_venv_activation() {
    for shell in &["zsh", "bash", "fish", "nu"] {
        let hook = generate_hook(shell).unwrap();
        assert!(
            hook.contains("--exports") || hook.contains("VEX_ORIGINAL_PATH"),
            "Hook for {} should refresh exported activation env",
            shell
        );
    }
}

#[test]
fn test_get_shell_config_path_nushell_alias() {
    let nu_path = get_shell_config_path("nushell");
    assert!(nu_path.is_ok());
    assert!(nu_path
        .unwrap()
        .to_string_lossy()
        .contains("config/nushell/config.nu"));
}

#[test]
fn test_is_vex_configured_with_different_formats() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file1 = NamedTempFile::new().unwrap();
    writeln!(file1, "eval '$(vex env zsh)'").unwrap();
    assert!(is_vex_configured(file1.path()).unwrap());

    let mut file2 = NamedTempFile::new().unwrap();
    writeln!(file2, "eval `vex env bash`").unwrap();
    assert!(is_vex_configured(file2.path()).unwrap());

    let mut file3 = NamedTempFile::new().unwrap();
    writeln!(file3, "  eval \"$(vex env zsh)\"  ").unwrap();
    assert!(is_vex_configured(file3.path()).unwrap());
}

#[test]
fn test_generate_zsh_hook_structure() {
    let hook = generate_hook("zsh").unwrap();
    assert!(hook.contains("add-zsh-hook"));
    assert!(hook.contains("chpwd"));
    assert!(hook.contains("__vex_use_if_found"));
}

#[test]
fn test_generate_bash_hook_structure() {
    let hook = generate_hook("bash").unwrap();
    assert!(hook.contains("PROMPT_COMMAND"));
    assert!(hook.contains("__vex_prompt_command"));
}

#[test]
fn test_generate_fish_hook_structure() {
    let hook = generate_hook("fish").unwrap();
    assert!(hook.contains("function"));
    assert!(hook.contains("on-variable PWD"));
    assert!(hook.contains("eval $exports"));
}

#[test]
fn test_generate_nushell_hook_structure() {
    let hook = generate_hook("nu").unwrap();
    assert!(hook.contains("def --env"));
    assert!(hook.contains("$env.config"));
    assert!(hook.contains("pre_prompt"));
}

#[test]
fn test_unsupported_shells() {
    let unsupported = vec!["powershell", "cmd", "tcsh", "csh", "ksh"];
    for shell in unsupported {
        let result = generate_hook(shell);
        assert!(result.is_err(), "Shell {} should be unsupported", shell);
    }
}
