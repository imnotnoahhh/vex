#!/bin/bash
# vex v1.0.1 Feature Test Script
# Usage: bash scripts/test-features.sh

PASS=0
FAIL=0

pass() { echo "  ✓ $1"; PASS=$((PASS+1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL+1)); }

check() {
    local desc="$1" cmd="$2" expect="$3"
    local output
    output=$(eval "$cmd" 2>&1) || true
    if echo "$output" | grep -q "$expect"; then
        pass "$desc"
    else
        fail "$desc (got: $output)"
    fi
}

check_not() {
    local desc="$1" cmd="$2" unexpected="$3"
    local output
    output=$(eval "$cmd" 2>&1) || true
    if echo "$output" | grep -q "$unexpected"; then
        fail "$desc (unexpectedly found: $unexpected)"
    else
        pass "$desc"
    fi
}

check_which() {
    local bin="$1"
    local path
    path=$(which "$bin" 2>/dev/null) || true
    if echo "$path" | grep -q "\.vex/bin"; then
        pass "which $bin → $path"
    else
        fail "which $bin not in ~/.vex/bin (got: $path)"
    fi
}

echo ""
echo "=== vex v1.0.1 Feature Tests ==="
echo ""

# ── 1. Basic ──────────────────────────────────────────────
echo "[ 1. Basic ]"
check "vex --version shows 1.0.1" "vex --version" "1.0.1"
# doctor may warn about network; just check no fatal errors
check_not "vex doctor has no errors" "vex doctor" "error"
echo ""

# ── 2. list-remote filters ────────────────────────────────
echo "[ 2. list-remote filters ]"
check "--filter latest returns 1 version" \
    "vex list-remote node --filter latest" "1 versions"
check "--filter lts returns LTS versions" \
    "vex list-remote node --filter lts" "LTS:"
check "--filter major sorted descending (25 first)" \
    "vex list-remote node --filter major" "25.0.0"
echo ""

# ── 3. install auto-switch ────────────────────────────────
echo "[ 3. install auto-switch ]"
check "install switches by default" \
    "vex install node@20.11.0" "Switched"
check "install --no-switch does not switch" \
    "vex install node@20.10.0 --no-switch" "To activate this version"
echo ""

# ── 4. version source display ─────────────────────────────
echo "[ 4. version source display ]"
# Ensure node 20.11.0 is active and set as global default
vex use node@20.11.0 > /dev/null 2>&1
vex global node@20.11.0 > /dev/null 2>&1
TMPDIR_TEST=$(mktemp -d)
# No local .tool-versions → should show Global default
check "global version shows 'Global default'" \
    "(cd $TMPDIR_TEST && vex current)" "Global default"
# Write MATCHING version to local .tool-versions → should show Project override
echo "node 20.11.0" > "$TMPDIR_TEST/.tool-versions"
check "project .tool-versions shows 'Project override'" \
    "(cd $TMPDIR_TEST && vex current)" "Project override"
rm -rf "$TMPDIR_TEST"
echo ""

# ── 5. Python venv ────────────────────────────────────────
echo "[ 5. Python venv ]"
PYDIR=$(mktemp -d)
check "python freeze without .venv: correct error" \
    "(cd $PYDIR && vex python freeze)" "No .venv found"
check "python sync without lock: correct error" \
    "(cd $PYDIR && vex python sync)" "No requirements.lock found"
(cd "$PYDIR" && vex python init > /dev/null 2>&1)
if [ -d "$PYDIR/.venv" ]; then pass "python init creates .venv"; else fail "python init creates .venv"; fi
(cd "$PYDIR" && vex python freeze > /dev/null 2>&1)
if [ -f "$PYDIR/requirements.lock" ]; then pass "python freeze creates requirements.lock"; else fail "python freeze creates requirements.lock"; fi
rm -rf "$PYDIR/.venv"
check "python sync restores from lock" \
    "(cd $PYDIR && vex python sync)" "Installing"
rm -rf "$PYDIR"
echo ""

# ── 6. Dynamic binary detection ───────────────────────────
echo "[ 6. Dynamic binary detection ]"
vex use java@21 > /dev/null 2>&1
if ls ~/.vex/bin/jnativescan > /dev/null 2>&1; then
    fail "Java 21 should NOT have jnativescan"
else
    pass "Java 21 does not have jnativescan"
fi
vex use java@25 > /dev/null 2>&1
check "Java 25 has jnativescan" "ls ~/.vex/bin/jnativescan" "jnativescan"

vex use node@25 > /dev/null 2>&1
if ls ~/.vex/bin/corepack > /dev/null 2>&1; then
    fail "Node 25 should NOT have corepack"
else
    pass "Node 25 does not have corepack"
fi
vex use node@20 > /dev/null 2>&1
check "Node 20 has corepack" "ls ~/.vex/bin/corepack" "corepack"
echo ""

# ── 7. Python binaries ────────────────────────────────────
echo "[ 7. Python binaries ]"
vex use python@3.12 > /dev/null 2>&1 || true
for bin in python3 pip3 python pip 2to3 idle3 pydoc3 python3-config; do
    if [ -e ~/.vex/bin/$bin ]; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
done
for bin in python3 pip3 python pip 2to3 pydoc3; do
    check_which "$bin"
done
check "python3 --version" "~/.vex/bin/python3 --version" "Python 3"
check "python3 -V"        "~/.vex/bin/python3 -V"        "Python 3"
check "python --version"  "~/.vex/bin/python --version"  "Python 3"
check "pip3 --version"    "~/.vex/bin/pip3 --version"    "pip"
check "pip --version"     "~/.vex/bin/pip --version"     "pip"
check "2to3 --help"       "~/.vex/bin/2to3 --help 2>&1"  "2to3"
check "python3-config --prefix" "~/.vex/bin/python3-config --prefix" "/"
echo ""

# ── 8. Rust binaries ──────────────────────────────────────
echo "[ 8. Rust binaries ]"
vex use rust@stable > /dev/null 2>&1 || true
for bin in rustc rustdoc cargo rustfmt cargo-fmt cargo-clippy clippy-driver rust-analyzer rust-gdb rust-lldb; do
    if [ -e ~/.vex/bin/$bin ]; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
done
for bin in rustc cargo rustfmt cargo-clippy rust-analyzer; do
    check_which "$bin"
done
check "rustc --version"         "rustc --version"         "rustc"
check "cargo --version"         "cargo --version"         "cargo"
check "rustfmt --version"       "rustfmt --version"       "rustfmt"
check "clippy-driver --version" "clippy-driver --version" "clippy"
check "rust-analyzer --version" "rust-analyzer --version" "rust-analyzer"
echo ""

# ── 9. Node binaries ──────────────────────────────────────
echo "[ 9. Node binaries ]"
vex use node@20 > /dev/null 2>&1 || true
for bin in node npm npx corepack; do
    if [ -e ~/.vex/bin/$bin ]; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
done
for bin in node npm npx; do
    check_which "$bin"
done
check "node --version" "node --version" "v"
check "npm --version"  "~/.vex/bin/node ~/.vex/toolchains/node/$(ls ~/.vex/toolchains/node/ | sort -V | tail -1)/lib/node_modules/npm/bin/npm-cli.js --version" "."
check "npx --version"  "~/.vex/bin/node ~/.vex/toolchains/node/$(ls ~/.vex/toolchains/node/ | sort -V | tail -1)/lib/node_modules/npm/bin/npx-cli.js --version" "."
echo ""

# ── 10. Go binaries ───────────────────────────────────────
echo "[ 10. Go binaries ]"
# Install go if not present (retry up to 3 times due to intermittent go.dev connectivity)
if ! ls ~/.vex/toolchains/go/ > /dev/null 2>&1; then
    echo "  (installing go@latest for test...)"
    for _attempt in 1 2 3; do
        vex install go@latest > /dev/null 2>&1 && break
        sleep 2
    done
fi
vex use go@latest > /dev/null 2>&1 || true
for bin in go gofmt; do
    if [ -e ~/.vex/bin/$bin ]; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
done
for bin in go gofmt; do
    check_which "$bin"
done
check "go version" "go version" "go"
echo ""

# ── 11. Java binaries ─────────────────────────────────────
echo "[ 11. Java binaries ]"
vex use java@21 > /dev/null 2>&1 || true
for bin in java javac jar javadoc javap jshell keytool jarsigner; do
    if [ -e ~/.vex/bin/$bin ]; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
done
for bin in java javac jar; do
    check_which "$bin"
done
check "java --version"  "java --version 2>&1 | head -1" "openjdk"
check "javac --version" "javac --version"               "javac"
echo ""

# ── 12. Concurrent install protection ─────────────────────
echo "[ 12. Concurrent install protection ]"
mkdir -p ~/.vex/locks
# Hold an exclusive flock on the lock file using a background Python process
# node@20.9.0 is not installed, so vex will attempt to install it and hit the lock
LOCK_VER="20.9.0"
LOCK_FILE="$HOME/.vex/locks/node-${LOCK_VER}.lock"
python3 -c "
import fcntl, time, sys, os
lock_file = os.path.expanduser('$LOCK_FILE')
os.makedirs(os.path.dirname(lock_file), exist_ok=True)
f = open(lock_file, 'w')
fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
sys.stdout.write('locked\n')
sys.stdout.flush()
time.sleep(30)
" &
LOCK_PID=$!
sleep 0.5
check "stale lock shows conflict message" \
    "vex install node@${LOCK_VER}" "Another vex process"
kill $LOCK_PID 2>/dev/null
wait $LOCK_PID 2>/dev/null
rm -f "$LOCK_FILE"
echo ""

# ── Summary ───────────────────────────────────────────────
echo "================================="
echo "  Passed: $PASS"
echo "  Failed: $FAIL"
echo "================================="
echo ""
if [ "$FAIL" -eq 0 ]; then
    echo "All tests passed! ✓"
    exit 0
else
    echo "Some tests failed. Check output above."
    exit 1
fi
