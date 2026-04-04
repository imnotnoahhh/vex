#!/bin/bash
# Focused shell integration smoke test for vex env hooks.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VEX_BIN="${VEX_BIN:-}"
if [ -z "$VEX_BIN" ]; then
    if [ -x "$ROOT_DIR/target/debug/vex" ]; then
        VEX_BIN="$ROOT_DIR/target/debug/vex"
    elif command -v vex >/dev/null 2>&1; then
        VEX_BIN="$(command -v vex)"
    else
        echo "Could not find vex. Set VEX_BIN=/path/to/vex or build target/debug/vex first." >&2
        exit 1
    fi
fi

BASE_PATH="$PATH"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/vex-shell-hooks.XXXXXX")"

PASS=0
FAIL=0

cleanup() {
    rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

pass() {
    echo "  ✓ $1"
    PASS=$((PASS + 1))
}

fail() {
    echo "  ✗ $1"
    FAIL=$((FAIL + 1))
}

setup_fake_workspace() {
    local home="$1"
    local workspace="$2"
    local project="$3"

    mkdir -p \
        "$home/.vex/toolchains/node/20.20.1/bin" \
        "$home/.vex/toolchains/node/25.8.0/bin" \
        "$workspace" \
        "$project/.venv/bin"

    cat > "$home/.vex/toolchains/node/20.20.1/bin/node" <<'EOF'
#!/bin/sh
echo workspace-node
EOF
    cat > "$home/.vex/toolchains/node/25.8.0/bin/node" <<'EOF'
#!/bin/sh
echo project-node
EOF
    chmod +x \
        "$home/.vex/toolchains/node/20.20.1/bin/node" \
        "$home/.vex/toolchains/node/25.8.0/bin/node"

    cat > "$workspace/.tool-versions" <<'EOF'
node 20
EOF

    cat > "$project/.tool-versions" <<'EOF'
node 25.8.0
EOF

    cat > "$project/.venv/bin/activate" <<'EOF'
VIRTUAL_ENV="$PWD/.venv"
export VIRTUAL_ENV
deactivate() { unset VIRTUAL_ENV; }
EOF
}

echo ""
echo "============================================================"
echo "vex shell hook smoke test"
echo "Text generation + real zsh/bash directory switching + .venv activation"
echo "============================================================"

echo ""
echo "[ hook generation ]"

zsh_hook="$TMP_ROOT/zsh-hook.txt"
"$VEX_BIN" env zsh > "$zsh_hook"
if grep -Fq '# vex shell integration' "$zsh_hook" \
    && grep -Fq 'add-zsh-hook chpwd' "$zsh_hook" \
    && grep -Fq '__vex_use_if_found' "$zsh_hook" \
    && grep -Fq '__vex_apply_exports' "$zsh_hook"; then
    pass "env zsh renders the current zsh hook"
else
    fail "env zsh did not render the expected zsh hook"
fi

bash_hook="$TMP_ROOT/bash-hook.txt"
"$VEX_BIN" env bash > "$bash_hook"
if grep -Fq '# vex shell integration' "$bash_hook" \
    && grep -Fq 'PROMPT_COMMAND' "$bash_hook" \
    && grep -Fq '__vex_prompt_command' "$bash_hook" \
    && grep -Fq '__vex_apply_exports' "$bash_hook"; then
    pass "env bash renders the current bash hook"
else
    fail "env bash did not render the expected bash hook"
fi

fish_hook="$TMP_ROOT/fish-hook.txt"
"$VEX_BIN" env fish > "$fish_hook"
if grep -Fq '# vex shell integration' "$fish_hook" \
    && grep -Fq 'function __vex_use_if_found' "$fish_hook" \
    && grep -Fq 'on-variable PWD' "$fish_hook" \
    && grep -Fq '__vex_apply_exports' "$fish_hook"; then
    pass "env fish renders the current fish hook"
else
    fail "env fish did not render the expected fish hook"
fi

nu_hook="$TMP_ROOT/nu-hook.txt"
"$VEX_BIN" env nu > "$nu_hook"
if grep -Fq '# vex shell integration' "$nu_hook" \
    && grep -Fq 'def --env __vex_use_if_found' "$nu_hook" \
    && grep -Fq 'pre_prompt' "$nu_hook" \
    && grep -Fq '__vex_apply_exports' "$nu_hook"; then
    pass "env nu renders the current nushell hook"
else
    fail "env nu did not render the expected nushell hook"
fi

invalid_shell="$TMP_ROOT/invalid-shell.txt"
if "$VEX_BIN" env csh >"$invalid_shell" 2>&1; then
    fail "env should reject unsupported shells"
elif grep -Fq 'Unsupported shell' "$invalid_shell"; then
    pass "env rejects unsupported shells clearly"
else
    fail "env reported an unexpected unsupported-shell error"
fi

echo ""
echo "[ real zsh workflow ]"

ZSH_HOME="$TMP_ROOT/zsh-home"
ZSH_WORKSPACE="$TMP_ROOT/zsh-workspace"
ZSH_PROJECT="$ZSH_WORKSPACE/project"
setup_fake_workspace "$ZSH_HOME" "$ZSH_WORKSPACE" "$ZSH_PROJECT"

zsh_result="$TMP_ROOT/zsh-workflow.txt"
HOME="$ZSH_HOME" PATH="$ROOT_DIR/target/debug:$BASE_PATH" VEX_BIN="$VEX_BIN" WORKSPACE="$ZSH_WORKSPACE" PROJECT="$ZSH_PROJECT" zsh -lc '
  vex() { "$VEX_BIN" "$@"; }
  eval "$("$VEX_BIN" env zsh)"
  cd "$WORKSPACE"
  workspace_node=$("$HOME/.vex/bin/node")
  cd "$PROJECT"
  project_node=$("$HOME/.vex/bin/node")
  project_venv=${VIRTUAL_ENV:-unset}
  cd "$WORKSPACE"
  after_node=$("$HOME/.vex/bin/node")
  after_venv=${VIRTUAL_ENV:-unset}
  printf "NODE1<<%s>>\nNODE2<<%s>>\nNODE3<<%s>>\nVENV1<<%s>>\nVENV2<<%s>>\n" \
    "$workspace_node" "$project_node" "$after_node" "$project_venv" "$after_venv"
' > "$zsh_result"

if grep -Fq 'NODE1<<workspace-node>>' "$zsh_result" \
    && grep -Fq 'NODE2<<project-node>>' "$zsh_result" \
    && grep -Fq 'NODE3<<workspace-node>>' "$zsh_result" \
    && grep -Fq 'VENV1<<' "$zsh_result" \
    && grep -Fq '/project/.venv>>' "$zsh_result" \
    && grep -Fq 'VENV2<<unset>>' "$zsh_result"; then
    pass "zsh hook auto-switches versions across directories and toggles .venv activation"
else
    fail "zsh hook did not auto-switch and toggle .venv as expected"
fi

echo ""
echo "[ real bash workflow ]"

BASH_HOME="$TMP_ROOT/bash-home"
BASH_WORKSPACE="$TMP_ROOT/bash-workspace"
BASH_PROJECT="$BASH_WORKSPACE/project"
setup_fake_workspace "$BASH_HOME" "$BASH_WORKSPACE" "$BASH_PROJECT"

bash_result="$TMP_ROOT/bash-workflow.txt"
HOME="$BASH_HOME" PATH="$ROOT_DIR/target/debug:$BASE_PATH" VEX_BIN="$VEX_BIN" WORKSPACE="$BASH_WORKSPACE" PROJECT="$BASH_PROJECT" bash -lc '
  vex() { "$VEX_BIN" "$@"; }
  eval "$("$VEX_BIN" env bash)"
  cd "$WORKSPACE"
  __vex_prompt_command
  workspace_node=$("$HOME/.vex/bin/node")
  cd "$PROJECT"
  __vex_prompt_command
  project_node=$("$HOME/.vex/bin/node")
  project_venv=${VIRTUAL_ENV:-unset}
  cd "$WORKSPACE"
  __vex_prompt_command
  after_node=$("$HOME/.vex/bin/node")
  after_venv=${VIRTUAL_ENV:-unset}
  printf "NODE1<<%s>>\nNODE2<<%s>>\nNODE3<<%s>>\nVENV1<<%s>>\nVENV2<<%s>>\n" \
    "$workspace_node" "$project_node" "$after_node" "$project_venv" "$after_venv"
' > "$bash_result"

if grep -Fq 'NODE1<<workspace-node>>' "$bash_result" \
    && grep -Fq 'NODE2<<project-node>>' "$bash_result" \
    && grep -Fq 'NODE3<<workspace-node>>' "$bash_result" \
    && grep -Fq 'VENV1<<' "$bash_result" \
    && grep -Fq '/project/.venv>>' "$bash_result" \
    && grep -Fq 'VENV2<<unset>>' "$bash_result"; then
    pass "bash hook auto-switches versions across directories and toggles .venv activation"
else
    fail "bash hook did not auto-switch and toggle .venv as expected"
fi

echo ""
echo "============================================================"
echo "Passed : $PASS"
echo "Failed : $FAIL"
echo "============================================================"

if [ "$FAIL" -ne 0 ]; then
    exit 1
fi
