pub(super) fn generate_nushell_hook() -> String {
    r#"# vex shell integration
$env.PATH = ($env.PATH | prepend $"($env.HOME)/.vex/bin")
$env.CARGO_HOME = $"($env.HOME)/.vex/cargo"

def --env __vex_use_if_found [] {
    mut dir = $env.PWD
    mut found = false

    # Search upward for version files
    while $dir != "" {
        if (
            ($dir | path join ".tool-versions" | path exists) or
            ($dir | path join ".node-version" | path exists) or
            ($dir | path join ".go-version" | path exists) or
            ($dir | path join ".java-version" | path exists) or
            ($dir | path join ".rust-toolchain" | path exists) or
            ($dir | path join ".python-version" | path exists)
        ) {
            vex use --auto | ignore
            $found = true
            break
        }
        $dir = ($dir | path dirname)
        if $dir == "/" {
            break
        }
    }

    # No project version found, fall back to global
    if (not $found) and (($env.HOME | path join ".vex" "tool-versions") | path exists) {
        vex use --auto | ignore
    }

    # Load environment variables after version switch
    __vex_load_env
}

def --env __vex_load_env [] {
    # Set JAVA_HOME if Java is active
    let java_current = ($env.HOME | path join ".vex" "current" "java")
    if ($java_current | path exists) and ($java_current | path type) == "symlink" {
        let java_path = (ls -l $java_current | get target.0)
        if (sys host | get name) == "Darwin" {
            $env.JAVA_HOME = ($java_path | path join "Contents" "Home")
        } else {
            $env.JAVA_HOME = $java_path
        }
    } else {
        hide-env JAVA_HOME
    }

    # Set GOROOT if Go is active
    let go_current = ($env.HOME | path join ".vex" "current" "go")
    if ($go_current | path exists) and ($go_current | path type) == "symlink" {
        $env.GOROOT = (ls -l $go_current | get target.0)
    } else {
        hide-env GOROOT
    }

    # Load project environment variables from .vex.toml
    let vex_toml = ".vex.toml"
    if ($vex_toml | path exists) {
        let content = (open $vex_toml)
        if ($content | get -i env | is-not-empty) {
            for item in ($content.env | transpose key value) {
                load-env {($item.key): ($item.value)}
            }
        }
    }
}

def --env __vex_activate_venv [] {
    let venv_activate = ($env.PWD | path join ".venv" "bin" "activate.nu")
    if ($venv_activate | path exists) {
        if ($env | get -i VIRTUAL_ENV | is-empty) or ($env.VIRTUAL_ENV != ($env.PWD | path join ".venv")) {
            source $venv_activate
        }
    }
}

$env.config = ($env.config | upsert hooks {
    pre_prompt: ($env.config.hooks.pre_prompt | append {||
        __vex_use_if_found
        __vex_activate_venv
    })
})

__vex_use_if_found
__vex_activate_venv
"#
    .to_string()
}
