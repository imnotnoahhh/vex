#!/bin/bash
# Focused smoke test for the newer management workflows.
# This script intentionally validates real command effects in isolated HOMEs.

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
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/vex-management-features.XXXXXX")"
LOCAL_HOME="$TMP_ROOT/local-home"
NETWORK_HOME="$TMP_ROOT/network-home"
PROJECT_DIR="$TMP_ROOT/project"
DOCTOR_PROJECT="$TMP_ROOT/doctor-project"

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

run_local() {
    HOME="$LOCAL_HOME" PATH="$LOCAL_HOME/.vex/bin:$BASE_PATH" "$VEX_BIN" "$@"
}

run_local_in() {
    local cwd="$1"
    shift
    (
        cd "$cwd"
        HOME="$LOCAL_HOME" PATH="$LOCAL_HOME/.vex/bin:$BASE_PATH" "$VEX_BIN" "$@"
    )
}

run_network() {
    HOME="$NETWORK_HOME" PATH="$NETWORK_HOME/.vex/bin:$BASE_PATH" "$VEX_BIN" "$@"
}

run_network_in() {
    local cwd="$1"
    shift
    (
        cd "$cwd"
        HOME="$NETWORK_HOME" PATH="$NETWORK_HOME/.vex/bin:$BASE_PATH" "$VEX_BIN" "$@"
    )
}

require_python_json() {
    local file="$1"
    local description="$2"
    local script="$3"
    if python3 - "$file" "$script" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text())
code = compile(sys.argv[2], "<assertion>", "exec")
namespace = {"payload": payload}
exec(code, namespace, namespace)
PY
    then
        pass "$description"
    else
        fail "$description"
    fi
}

write_fake_node() {
    local path="$1"
    local version="$2"
    cat > "$path" <<EOF
#!/bin/sh
printf 'node-version=%s\n' "$version"
printf 'exec-context=%s\n' "\${EXEC_CONTEXT:-unset}"
printf 'cwd=%s\n' "\$PWD"
if [ "\$#" -gt 0 ]; then
  printf 'args=%s\n' "\$*"
fi
EOF
    chmod +x "$path"
}

echo ""
echo "============================================================"
echo "vex management feature smoke test"
echo "JSON + outdated + upgrade --all + prune/gc + doctor + .vex.toml + exec/run"
echo "============================================================"

mkdir -p "$LOCAL_HOME" "$NETWORK_HOME" "$PROJECT_DIR" "$DOCTOR_PROJECT"

run_local init --shell zsh >/dev/null

echo ""
echo "[ init and template workflows ]"

template_list="$TMP_ROOT/template-list.txt"
run_local init --list-templates > "$template_list"
if grep -Fq 'node-typescript' "$template_list" \
    && grep -Fq 'go-service' "$template_list" \
    && grep -Fq 'java-basic' "$template_list" \
    && grep -Fq 'rust-cli' "$template_list" \
    && grep -Fq 'python-venv' "$template_list"; then
    pass "init --list-templates reports all built-in templates"
else
    fail "init --list-templates did not report the expected templates"
fi

invalid_init_args="$TMP_ROOT/init-invalid-args.txt"
if run_local init --list-templates --dry-run >"$invalid_init_args" 2>&1; then
    fail "init rejects mutually exclusive --list-templates --dry-run flags"
elif grep -Fq "cannot be used with '--dry-run'" "$invalid_init_args"; then
    pass "init rejects mutually exclusive --list-templates --dry-run flags"
else
    fail "init reported an unexpected error for --list-templates --dry-run"
fi

INIT_PREVIEW_HOME="$TMP_ROOT/init-preview-home"
mkdir -p "$INIT_PREVIEW_HOME"
shell_preview="$TMP_ROOT/init-shell-preview.txt"
HOME="$INIT_PREVIEW_HOME" PATH="$BASE_PATH" "$VEX_BIN" init --shell bash --dry-run > "$shell_preview"
if grep -Fq 'Would create' "$shell_preview" \
    && grep -Fq 'Would append to' "$shell_preview" \
    && [ ! -d "$INIT_PREVIEW_HOME/.vex" ] \
    && [ ! -f "$INIT_PREVIEW_HOME/.bash_profile" ]; then
    pass "init --shell bash --dry-run previews home setup without writing files"
else
    fail "init --shell bash --dry-run did not behave like a pure preview"
fi

template_dry_run_dir="$TMP_ROOT/template-dry-run"
mkdir -p "$template_dry_run_dir"
template_dry_run="$TMP_ROOT/template-dry-run.txt"
run_local_in "$template_dry_run_dir" init --template python-venv --dry-run > "$template_dry_run"
if grep -Fq 'No files were written (--dry-run).' "$template_dry_run" \
    && [ ! -e "$template_dry_run_dir/.tool-versions" ] \
    && [ ! -e "$template_dry_run_dir/.vex.toml" ] \
    && [ ! -d "$template_dry_run_dir/src" ]; then
    pass "template --dry-run previews python-venv without writing any files"
else
    fail "template --dry-run wrote files or omitted the dry-run summary"
fi

template_conflict_dir="$TMP_ROOT/template-conflict"
mkdir -p "$template_conflict_dir"
cat > "$template_conflict_dir/.tool-versions" <<'EOF'
node 20.11.0
EOF
template_conflict_out="$TMP_ROOT/template-conflict.txt"
if run_local_in "$template_conflict_dir" init --template python-venv >"$template_conflict_out" 2>&1; then
    fail "strict template mode should reject existing conflicting files"
elif grep -Fq 'Template could not be applied because these files already exist' "$template_conflict_out" \
    && grep -Fq '.tool-versions' "$template_conflict_out"; then
    pass "strict template mode reports conflicting existing files"
else
    fail "strict template mode did not report the expected conflict details"
fi

template_add_only_dir="$TMP_ROOT/template-add-only"
mkdir -p "$template_add_only_dir"
cat > "$template_add_only_dir/.tool-versions" <<'EOF'
node 20.11.0
EOF
cat > "$template_add_only_dir/.gitignore" <<'EOF'
# existing ignore
EOF
template_add_only_out="$TMP_ROOT/template-add-only.txt"
run_local_in "$template_add_only_dir" init --template python-venv --add-only > "$template_add_only_out"
if grep -Fq 'merge' "$template_add_only_out" \
    && grep -Fq 'python 3.12' "$template_add_only_dir/.tool-versions" \
    && grep -Fq '# existing ignore' "$template_add_only_dir/.gitignore" \
    && grep -Fq '.venv/' "$template_add_only_dir/.gitignore" \
    && [ -f "$template_add_only_dir/.vex.toml" ] \
    && [ -f "$template_add_only_dir/src/main.py" ] \
    && [ -f "$template_add_only_dir/tests/test_main.py" ]; then
    pass "template --add-only safely merges .tool-versions/.gitignore and creates missing files"
else
    fail "template --add-only did not merge and create files as expected"
fi

echo ""
echo "[ alias workflows ]"

alias_set_out="$TMP_ROOT/alias-set.txt"
run_local alias set node prod 20.11.0 > "$alias_set_out"
if grep -Fq 'Set global alias: node@prod -> 20.11.0' "$alias_set_out"; then
    pass "alias set stores a global alias"
else
    fail "alias set did not report the expected global alias change"
fi

alias_list_out="$TMP_ROOT/alias-list.txt"
run_local alias list node > "$alias_list_out"
if grep -Fq 'Global aliases' "$alias_list_out" \
    && grep -Fq 'prod' "$alias_list_out" \
    && grep -Fq '20.11.0' "$alias_list_out"; then
    pass "alias list shows the stored global alias"
else
    fail "alias list did not show the expected global alias"
fi

alias_delete_out="$TMP_ROOT/alias-delete.txt"
run_local alias delete node prod > "$alias_delete_out"
if grep -Fq 'Deleted global alias: node@prod' "$alias_delete_out" \
    && { [ ! -f "$LOCAL_HOME/.vex/aliases.toml" ] || ! grep -Fq 'prod' "$LOCAL_HOME/.vex/aliases.toml"; }; then
    pass "alias delete removes the stored alias from disk"
else
    fail "alias delete did not remove the stored alias"
fi

mkdir -p "$LOCAL_HOME/.vex/toolchains/node/20.20.1/bin"
mkdir -p "$LOCAL_HOME/.vex/toolchains/node/25.8.0/bin"
mkdir -p "$LOCAL_HOME/.vex/toolchains/go/9.9.9/bin"
write_fake_node "$LOCAL_HOME/.vex/toolchains/node/20.20.1/bin/node" "20.20.1"
write_fake_node "$LOCAL_HOME/.vex/toolchains/node/25.8.0/bin/node" "25.8.0"
cat > "$LOCAL_HOME/.vex/toolchains/go/9.9.9/bin/go" <<'EOF'
#!/bin/sh
echo go-unused
EOF
chmod +x "$LOCAL_HOME/.vex/toolchains/go/9.9.9/bin/go"

cat > "$LOCAL_HOME/.vex/tool-versions" <<'EOF'
node 20.20.1
EOF
ln -s "$LOCAL_HOME/.vex/toolchains/node/20.20.1" "$LOCAL_HOME/.vex/current/node"

cat > "$PROJECT_DIR/.tool-versions" <<'EOF'
node 25.8.0
EOF
cat > "$PROJECT_DIR/.vex.toml" <<'EOF'
[behavior]
default_shell = "/bin/sh"

[env]
EXEC_CONTEXT = "from-project"

[commands]
show = "node"
EOF

current_json="$TMP_ROOT/current.json"
run_local current --json > "$current_json"
require_python_json "$current_json" "current --json reports the active node version" \
    "assert any(entry['tool'] == 'node' and entry['version'] == '20.20.1' for entry in payload['tools'])"

installed_json="$TMP_ROOT/list-node.json"
run_local list node --json > "$installed_json"
require_python_json "$installed_json" "list node --json includes both installed versions" \
    "versions = {entry['version'] for entry in payload['versions']}; assert {'20.20.1', '25.8.0'} <= versions; assert payload['current_version'] == '20.20.1'"

exec_output="$TMP_ROOT/exec.txt"
run_local_in "$PROJECT_DIR" exec -- node > "$exec_output"
if grep -Fq 'node-version=25.8.0' "$exec_output" \
    && grep -Fq 'exec-context=from-project' "$exec_output" \
    && grep -Eq 'cwd=.*/project$' "$exec_output"; then
    pass "exec uses project resolution and .vex.toml env without needing a global switch"
else
    fail "exec did not resolve the project node/tool env correctly (got: $(tr '\n' '|' < "$exec_output"))"
fi

if [ "$(basename "$(readlink "$LOCAL_HOME/.vex/current/node")")" = "20.20.1" ]; then
    pass "exec does not mutate the globally active version"
else
    fail "exec unexpectedly changed ~/.vex/current/node"
fi

run_output="$TMP_ROOT/run.txt"
run_local_in "$PROJECT_DIR" run show -- from-run > "$run_output"
if grep -Fq 'node-version=25.8.0' "$run_output" \
    && grep -Fq 'exec-context=from-project' "$run_output" \
    && grep -Eq 'cwd=.*/project$' "$run_output" \
    && grep -Fq 'args=from-run' "$run_output"; then
    pass "run executes the .vex.toml task from the project root with forwarded args"
else
    fail "run did not execute the project task as expected (got: $(tr '\n' '|' < "$run_output"))"
fi

echo ""
echo "[ install, sync, source, and frozen workflows ]"

project_install_out="$TMP_ROOT/project-install.txt"
run_local_in "$PROJECT_DIR" install > "$project_install_out"
if grep -Fq 'node@25.8.0 already installed, skipping.' "$project_install_out"; then
    pass "install with no specs reads the current project .tool-versions"
else
    fail "install with no specs did not use the current project .tool-versions"
fi

fuzzy_project_dir="$TMP_ROOT/fuzzy-project"
mkdir -p "$fuzzy_project_dir"
cat > "$fuzzy_project_dir/.tool-versions" <<'EOF'
node 20
EOF
fuzzy_install_out="$TMP_ROOT/fuzzy-install.txt"
run_local_in "$fuzzy_project_dir" install > "$fuzzy_install_out"
if grep -Fq 'node@20.20.1 already installed, skipping.' "$fuzzy_install_out"; then
    pass "install resolves fuzzy project version pins before checking installed toolchains"
else
    fail "install did not resolve fuzzy project version pins as expected"
fi

project_install_offline_out="$TMP_ROOT/project-install-offline.txt"
run_local_in "$PROJECT_DIR" install --offline > "$project_install_offline_out"
if grep -Fq 'node@25.8.0 already installed, skipping.' "$project_install_offline_out"; then
    pass "install --offline succeeds when the requested toolchain is already available locally"
else
    fail "install --offline did not reuse the already installed toolchain"
fi

project_sync_out="$TMP_ROOT/project-sync.txt"
run_local_in "$PROJECT_DIR" sync > "$project_sync_out"
if grep -Fq 'Sync Summary:' "$project_sync_out" \
    && grep -Fq 'node' "$project_sync_out" \
    && grep -Fq '25.8.0' "$project_sync_out" \
    && grep -Fq '(already installed)' "$project_sync_out"; then
    pass "sync with no explicit source uses the current project version files"
else
    fail "sync with no explicit source did not report the project version file results"
fi

custom_version_file="$PROJECT_DIR/custom.versions"
cat > "$custom_version_file" <<'EOF'
node 20.20.1
EOF
install_from_source_out="$TMP_ROOT/install-from-source.txt"
run_local_in "$PROJECT_DIR" install --from custom.versions > "$install_from_source_out"
if grep -Fq 'Sync Summary:' "$install_from_source_out" \
    && grep -Fq 'node' "$install_from_source_out" \
    && grep -Fq '20.20.1' "$install_from_source_out" \
    && grep -Fq '(already installed)' "$install_from_source_out"; then
    pass "install --from local version file reads explicit sources relative to the cwd"
else
    fail "install --from local version file did not report the expected install summary"
fi

team_config_file="$PROJECT_DIR/vex-config.toml"
cat > "$team_config_file" <<'EOF'
version = 1

[tools]
node = "20.20.1"
EOF
sync_from_team_config_out="$TMP_ROOT/sync-from-team-config.txt"
run_local_in "$PROJECT_DIR" sync --from vex-config.toml > "$sync_from_team_config_out"
if grep -Fq 'Sync Summary:' "$sync_from_team_config_out" \
    && grep -Fq 'node' "$sync_from_team_config_out" \
    && grep -Fq '25.8.0' "$sync_from_team_config_out" \
    && grep -Fq '(already installed)' "$sync_from_team_config_out"; then
    pass "sync --from vex-config.toml keeps the project-local version ahead of the remote baseline"
else
    fail "sync --from vex-config.toml did not preserve local-over-remote version precedence"
fi

team_repo="$TMP_ROOT/team-config-repo"
mkdir -p "$team_repo"
git init --quiet "$team_repo"
git -C "$team_repo" config user.email codex@example.com
git -C "$team_repo" config user.name Codex
cat > "$team_repo/vex-config.toml" <<'EOF'
version = 1

[tools]
node = "20.20.1"
EOF
git -C "$team_repo" add vex-config.toml
git -C "$team_repo" commit -m "Add team config" --quiet
sync_from_git_out="$TMP_ROOT/sync-from-git.txt"
run_local_in "$PROJECT_DIR" sync --from "$team_repo" > "$sync_from_git_out"
if grep -Fq 'Sync Summary:' "$sync_from_git_out" \
    && grep -Fq 'node' "$sync_from_git_out" \
    && grep -Fq '25.8.0' "$sync_from_git_out" \
    && grep -Fq '(already installed)' "$sync_from_git_out"; then
    pass "sync --from local git repo also honors local-over-remote version precedence"
else
    fail "sync --from local git repo did not preserve local-over-remote version precedence"
fi

auto_use_out="$TMP_ROOT/use-auto.txt"
run_local_in "$PROJECT_DIR" use --auto > "$auto_use_out"
auto_use_node_out="$TMP_ROOT/use-auto-node.txt"
"$LOCAL_HOME/.vex/bin/node" > "$auto_use_node_out"
if grep -Fq 'Switched to node@25.8.0' "$auto_use_out" \
    && [ "$(basename "$(readlink "$LOCAL_HOME/.vex/current/node")")" = "25.8.0" ] \
    && grep -Fq 'node-version=25.8.0' "$auto_use_node_out"; then
    pass "use --auto switches to the project-resolved version and refreshes the active symlink"
else
    fail "use --auto did not refresh the active project version as expected"
fi

lock_project="$TMP_ROOT/lock-project"
mkdir -p "$lock_project"
cat > "$lock_project/.tool-versions" <<'EOF'
node 20.20.1
EOF
lock_out="$TMP_ROOT/lock.txt"
run_local_in "$lock_project" lock > "$lock_out"
if grep -Fq 'Lockfile generated:' "$lock_out" \
    && grep -Fq 'version = "20.20.1"' "$lock_project/.tool-versions.lock"; then
    pass "lock generates a lockfile from the current .tool-versions"
else
    fail "lock did not generate the expected .tool-versions.lock file"
fi

sync_frozen_out="$TMP_ROOT/sync-frozen.txt"
run_local_in "$lock_project" sync --frozen > "$sync_frozen_out"
if grep -Fq 'Sync Summary:' "$sync_frozen_out" \
    && grep -Fq 'node' "$sync_frozen_out" \
    && grep -Fq '20.20.1' "$sync_frozen_out" \
    && grep -Fq '(already installed)' "$sync_frozen_out"; then
    pass "sync --frozen reads the generated lockfile and enforces the locked version"
else
    fail "sync --frozen did not use the generated lockfile"
fi

install_frozen_out="$TMP_ROOT/install-frozen.txt"
run_local_in "$lock_project" install --frozen > "$install_frozen_out"
if grep -Fq 'node@20.20.1 already installed, skipping.' "$install_frozen_out"; then
    pass "install --frozen respects the generated lockfile"
else
    fail "install --frozen did not use the generated lockfile"
fi

cat > "$DOCTOR_PROJECT/.vex.toml" <<'EOF'
[network]
proxy = "://bad-proxy"

[mirrors]
node = "not-a-url"
EOF

doctor_json="$TMP_ROOT/doctor.json"
run_local_in "$DOCTOR_PROJECT" doctor --json > "$doctor_json"
require_python_json "$doctor_json" "doctor --json reports invalid effective settings from project config" \
    "check = next(item for item in payload['checks'] if item['id'] == 'effective_settings'); assert check['status'] == 'warn'; details = '\\n'.join(check['details']); assert 'Invalid proxy URL' in details and 'Invalid mirror for node' in details"

mkdir -p "$LOCAL_HOME/.vex/cache"
printf 'stale-cache' > "$LOCAL_HOME/.vex/cache/node-stale.tar.gz"
mkdir -p "$LOCAL_HOME/.vex/locks"
printf 'stale-lock' > "$LOCAL_HOME/.vex/locks/old.lock"
python3 - "$LOCAL_HOME/.vex/locks/old.lock" <<'PY'
import os
import sys
import time

target = sys.argv[1]
old = time.time() - 7200
os.utime(target, (old, old))
PY

prune_dry_run="$TMP_ROOT/prune-dry-run.txt"
run_local_in "$PROJECT_DIR" prune --dry-run > "$prune_dry_run"
if grep -Fq 'node-stale.tar.gz' "$prune_dry_run" \
    && grep -Fq 'old.lock' "$prune_dry_run" \
    && grep -Fq '/go/9.9.9' "$prune_dry_run"; then
    pass "prune --dry-run identifies cache, stale locks, and unused toolchains"
else
    fail "prune --dry-run did not report the expected removal candidates"
fi

gc_dry_run="$TMP_ROOT/gc-dry-run.txt"
run_local_in "$PROJECT_DIR" gc --dry-run > "$gc_dry_run"
if grep -q 'vex prune --dry-run' "$gc_dry_run"; then
    pass "gc behaves as an alias of prune"
else
    fail "gc --dry-run did not behave like prune --dry-run"
fi

run_local_in "$PROJECT_DIR" prune > /dev/null
if [ -d "$LOCAL_HOME/.vex/toolchains/node/20.20.1" ] \
    && [ -d "$LOCAL_HOME/.vex/toolchains/node/25.8.0" ] \
    && [ ! -d "$LOCAL_HOME/.vex/toolchains/go/9.9.9" ] \
    && [ ! -f "$LOCAL_HOME/.vex/cache/node-stale.tar.gz" ] \
    && [ ! -f "$LOCAL_HOME/.vex/locks/old.lock" ]; then
    pass "prune removes unmanaged state while retaining active/global/project toolchains"
else
    fail "prune did not preserve or remove the expected paths"
fi

list_remote_offline_out="$TMP_ROOT/list-remote-offline.txt"
if run_local list-remote node --offline >"$list_remote_offline_out" 2>&1; then
    fail "list-remote --offline should fail without cached version data"
elif grep -Fq 'No cached version data available for node in offline mode' "$list_remote_offline_out"; then
    pass "list-remote --offline reports the missing cache requirement clearly"
else
    fail "list-remote --offline did not report the expected offline cache error"
fi

tui_out="$TMP_ROOT/tui.txt"
if run_local tui >"$tui_out" 2>&1; then
    fail "tui should refuse to start without an interactive terminal"
elif grep -Fq 'TUI requires an interactive terminal' "$tui_out"; then
    pass "tui fails clearly in non-interactive environments"
else
    fail "tui reported an unexpected non-interactive error"
fi

run_network init --shell zsh >/dev/null

remote_json="$TMP_ROOT/list-remote-python.json"
run_network list-remote python --json > "$remote_json"
require_python_json "$remote_json" "list-remote python --json returns structured remote data" \
    "assert payload['tool'] == 'python'; assert payload['total'] > 0; assert payload['versions']; first = payload['versions'][0]; assert 'version' in first and 'label' in first"

python_versions="$TMP_ROOT/python-upgrade-versions.txt"
python3 - "$remote_json" > "$python_versions" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1]))
versions = payload["versions"]
stable_versions = [
    entry["version"]
    for entry in versions
    if entry.get("label") in {"bugfix", "security", "end_of_life"}
]

if len(stable_versions) < 2:
    raise SystemExit("Need at least two stable Python versions for outdated/upgrade smoke testing")

print(stable_versions[0])
print(stable_versions[1])
PY
LATEST_PYTHON_VERSION="$(sed -n '1p' "$python_versions")"
OLDER_PYTHON_VERSION="$(sed -n '2p' "$python_versions")"

run_network install "python@${OLDER_PYTHON_VERSION}" --no-switch > /dev/null
run_network global "python@${OLDER_PYTHON_VERSION}" > /dev/null

outdated_json="$TMP_ROOT/outdated.json"
run_network outdated --json > "$outdated_json"
require_python_json "$outdated_json" "outdated --json marks an older managed Python as outdated" \
    "entry = next(item for item in payload['entries'] if item['tool'] == 'python'); assert entry['current_version'] == '${OLDER_PYTHON_VERSION}'; assert entry['status'] == 'outdated'; assert entry['latest_version'] == '${LATEST_PYTHON_VERSION}'"

run_network upgrade --all > /dev/null
upgrade_json="$TMP_ROOT/current-after-upgrade.json"
run_network current --json > "$upgrade_json"
require_python_json "$upgrade_json" "upgrade --all updates the active Python pin away from the older version" \
    "entry = next(item for item in payload['tools'] if item['tool'] == 'python'); assert entry['version'] == '${LATEST_PYTHON_VERSION}'"

if grep -q '^python ' "$NETWORK_HOME/.vex/tool-versions" \
    && grep -Fq "python ${LATEST_PYTHON_VERSION}" "$NETWORK_HOME/.vex/tool-versions"; then
    pass "upgrade --all rewrites the global tool-versions pin"
else
    fail "upgrade --all did not rewrite the global tool-versions pin"
fi

echo ""
echo "============================================================"
echo "Passed : $PASS"
echo "Failed : $FAIL"
echo "============================================================"

if [ "$FAIL" -ne 0 ]; then
    exit 1
fi
