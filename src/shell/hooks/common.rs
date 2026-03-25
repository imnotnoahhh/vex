pub(super) fn vex_hook_function() -> &'static str {
    r#"
__vex_use_if_found() {
    local dir="$PWD"
    local found=0

    # Search upward for version files
    while [ "$dir" != "" ]; do
        if [ -f "$dir/.tool-versions" ] || \
           [ -f "$dir/.node-version" ] || \
           [ -f "$dir/.go-version" ] || \
           [ -f "$dir/.java-version" ] || \
           [ -f "$dir/.rust-toolchain" ] || \
           [ -f "$dir/.python-version" ]; then
            vex use --auto 2>/dev/null
            found=1
            break
        fi
        dir="${dir%/*}"
    done

    # No project version found, fall back to global
    if [ $found -eq 0 ] && [ -f "$HOME/.vex/tool-versions" ]; then
        vex use --auto 2>/dev/null
    fi

    # Load environment variables after version switch
    __vex_load_env
}

__vex_load_env() {
    # Set JAVA_HOME if Java is active
    if [ -L "$HOME/.vex/current/java" ]; then
        local java_path="$(readlink "$HOME/.vex/current/java")"
        if [ "$(uname)" = "Darwin" ]; then
            export JAVA_HOME="$java_path/Contents/Home"
        else
            export JAVA_HOME="$java_path"
        fi
    else
        unset JAVA_HOME
    fi

    # Set GOROOT if Go is active
    if [ -L "$HOME/.vex/current/go" ]; then
        export GOROOT="$(readlink "$HOME/.vex/current/go")"
    else
        unset GOROOT
    fi

    # Load project environment variables from .vex.toml
    if [ -f ".vex.toml" ]; then
        # Parse [env] section and export variables
        # This is a simple implementation - only handles basic key="value" format
        local in_env_section=0
        while IFS= read -r line; do
            if [[ "$line" =~ ^\[env\] ]]; then
                in_env_section=1
                continue
            elif [[ "$line" =~ ^\[ ]]; then
                in_env_section=0
                continue
            fi

            if [ $in_env_section -eq 1 ] && [[ "$line" =~ ^[A-Z_][A-Z0-9_]*[[:space:]]*=[[:space:]]*\".*\"$ ]]; then
                # Extract key and value
                local key="${line%%=*}"
                key="${key// /}"  # Remove spaces
                local value="${line#*=}"
                value="${value#*\"}"  # Remove leading quote
                value="${value%\"*}"  # Remove trailing quote
                export "$key=$value"
            fi
        done < ".vex.toml"
    fi
}

__vex_activate_venv() {
    if [ -f "$PWD/.venv/bin/activate" ]; then
        if [ -z "$VIRTUAL_ENV" ] || [ "$VIRTUAL_ENV" != "$PWD/.venv" ]; then
            VIRTUAL_ENV_DISABLE_PROMPT=1 source "$PWD/.venv/bin/activate"
        fi
    elif [ -n "$VIRTUAL_ENV" ]; then
        deactivate 2>/dev/null || true
    fi
}
"#
}
