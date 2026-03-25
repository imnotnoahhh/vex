pub(super) fn generate_fish_hook() -> String {
    r#"# vex shell integration
set -gx PATH $HOME/.vex/bin $PATH
set -gx CARGO_HOME $HOME/.vex/cargo

function __vex_use_if_found
    set -l dir $PWD
    set -l found 0

    # Search upward for version files
    while test "$dir" != ""
        if test -f "$dir/.tool-versions"; or \
           test -f "$dir/.node-version"; or \
           test -f "$dir/.go-version"; or \
           test -f "$dir/.java-version"; or \
           test -f "$dir/.rust-toolchain"; or \
           test -f "$dir/.python-version"
            vex use --auto 2>/dev/null
            set found 1
            break
        end
        set dir (string replace -r '/[^/]*$' '' "$dir")
    end

    # No project version found, fall back to global
    if test $found -eq 0; and test -f "$HOME/.vex/tool-versions"
        vex use --auto 2>/dev/null
    end

    # Load environment variables after version switch
    __vex_load_env
end

function __vex_load_env
    # Set JAVA_HOME if Java is active
    if test -L "$HOME/.vex/current/java"
        set -l java_path (readlink "$HOME/.vex/current/java")
        if test (uname) = "Darwin"
            set -gx JAVA_HOME "$java_path/Contents/Home"
        else
            set -gx JAVA_HOME "$java_path"
        end
    else
        set -e JAVA_HOME 2>/dev/null
    end

    # Set GOROOT if Go is active
    if test -L "$HOME/.vex/current/go"
        set -gx GOROOT (readlink "$HOME/.vex/current/go")
    else
        set -e GOROOT 2>/dev/null
    end

    # Load project environment variables from .vex.toml
    if test -f ".vex.toml"
        set -l in_env_section 0
        for line in (cat ".vex.toml")
            if string match -q -r '^\[env\]' "$line"
                set in_env_section 1
                continue
            else if string match -q -r '^\[' "$line"
                set in_env_section 0
                continue
            end

            if test $in_env_section -eq 1
                if string match -q -r '^[A-Z_][A-Z0-9_]*\s*=\s*".*"$' "$line"
                    set -l key (string replace -r '=.*' '' "$line" | string trim)
                    set -l value (string replace -r '^[^=]*=\s*"' '' "$line" | string replace -r '".*' '')
                    set -gx $key $value
                end
            end
        end
    end
end

function __vex_activate_venv
    if test -f "$PWD/.venv/bin/activate.fish"
        if test -z "$VIRTUAL_ENV"; or test "$VIRTUAL_ENV" != "$PWD/.venv"
            source "$PWD/.venv/bin/activate.fish"
        end
    else if set -q VIRTUAL_ENV
        deactivate 2>/dev/null; or true
    end
end

function __vex_on_pwd --on-variable PWD
    __vex_use_if_found
    __vex_activate_venv
end

__vex_use_if_found
__vex_activate_venv
"#
    .to_string()
}
