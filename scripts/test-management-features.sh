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

run_network init --shell zsh >/dev/null

remote_json="$TMP_ROOT/list-remote-python.json"
run_network list-remote python --json > "$remote_json"
require_python_json "$remote_json" "list-remote python --json returns structured remote data" \
    "assert payload['tool'] == 'python'; assert payload['total'] > 0; assert payload['versions']; first = payload['versions'][0]; assert 'version' in first and 'label' in first"

python_versions="$TMP_ROOT/python-upgrade-versions.txt"
python3 - "$remote_json" <<'PY' > "$python_versions"
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
