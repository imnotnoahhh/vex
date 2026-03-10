#!/bin/bash
# vex v1.0.0 Feature Test Script
# Run this after installing vex to verify all features work correctly.
# Usage: bash scripts/test-features.sh

set -e

PASS=0
FAIL=0

pass() { echo "  ✓ $1"; PASS=$((PASS+1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL+1)); }

check() {
    local desc="$1"
    local cmd="$2"
    local expect="$3"
    local output
    output=$(eval "$cmd" 2>&1) || true
    if echo "$output" | grep -q "$expect"; then
        pass "$desc"
    else
        fail "$desc (got: $output)"
    fi
}

echo ""
echo "=== vex v1.0.0 Feature Tests ==="
echo ""

# ── 1. Basic ──────────────────────────────────────────────
echo "[ 1. Basic ]"
check "vex --version shows 1.0.0" "vex --version" "1.0.0"
check "vex doctor passes" "vex doctor" "All checks passed"
echo ""

# ── 2. list-remote filters ────────────────────────────────
echo "[ 2. list-remote filters ]"
check "--filter latest returns 1 version" \
    "vex list-remote node --filter latest" "Total: 1 versions"
check "--filter lts returns only LTS versions" \
    "vex list-remote node --filter lts" "LTS:"
check "--filter major returns sorted versions (25 first)" \
    "vex list-remote node --filter major" "25.0.0"
check "--filter major is sorted descending" \
    "vex list-remote node --filter major | head -5" "25.0.0"
echo ""

# ── 3. install auto-switch ────────────────────────────────
echo "[ 3. install auto-switch ]"
check "install switches by default" \
    "vex install node@20.11.0 2>&1" "Switched"
check "install --no-switch does not switch" \
    "vex install node@20.10.0 --no-switch 2>&1" "To activate this version"
echo ""

# ── 4. version source display ─────────────────────────────
echo "[ 4. version source display ]"
TMPDIR_TEST=$(mktemp -d)
vex global node@20.11.0 > /dev/null 2>&1
check "global version shows 'Global default'" \
    "cd $TMPDIR_TEST && vex current" "Global default"
echo "node 22.0.0" > "$TMPDIR_TEST/.tool-versions"
check "project .tool-versions shows 'Project override'" \
    "cd $TMPDIR_TEST && vex current" "Project override"
rm -rf "$TMPDIR_TEST"
echo ""

# ── 5. Python venv ────────────────────────────────────────
echo "[ 5. Python venv ]"
PYDIR=$(mktemp -d)
check "python freeze without .venv shows correct error" \
    "cd $PYDIR && vex python freeze" "No .venv found"
check "python sync without requirements.lock shows correct error" \
    "cd $PYDIR && vex python sync" "No requirements.lock found"
check "python init creates .venv" \
    "cd $PYDIR && vex python init && ls $PYDIR" ".venv"
check "python freeze creates requirements.lock" \
    "cd $PYDIR && vex python freeze && ls $PYDIR" "requirements.lock"
check "python sync restores from requirements.lock" \
    "cd $PYDIR && rm -rf .venv && vex python sync" "Installing"
rm -rf "$PYDIR"
echo ""

# ── 6. Dynamic binary detection ───────────────────────────
echo "[ 6. Dynamic binary detection ]"
check "Java 21 does not have jnativescan" \
    "vex use java@21 > /dev/null 2>&1 && ls ~/.vex/bin/ | grep -c jnativescan || echo 0" "0"
check "Java 25 has jnativescan" \
    "vex use java@25 > /dev/null 2>&1 && ls ~/.vex/bin/ | grep jnativescan" "jnativescan"
check "Node 25 does not have corepack" \
    "vex use node@25 > /dev/null 2>&1 && (ls ~/.vex/bin/corepack 2>/dev/null || echo 'not found')" "not found"
check "Node 20 has corepack" \
    "vex use node@20 > /dev/null 2>&1 && ls ~/.vex/bin/corepack" "corepack"
echo ""

# ── 7. Python binary completeness + execution ─────────────
echo "[ 7. Python binaries ]"
vex use python@3.12 > /dev/null 2>&1 || true
# Check symlinks exist
for bin in python3 pip3 python pip 2to3 idle3 pydoc3 python3-config; do
    if ls ~/.vex/bin/$bin > /dev/null 2>&1; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
done
# Check executables actually work
check "python3 --version works"   "python3 --version"      "Python 3"
check "python3 -V works"          "python3 -V"             "Python 3"
check "python --version works"    "python --version"       "Python 3"
check "pip3 --version works"      "pip3 --version"         "pip"
check "pip --version works"       "pip --version"          "pip"
check "2to3 --version works"      "2to3 --version"         "2to3"
check "pydoc3 outputs help"       "pydoc3 pydoc | head -1" "pydoc"
check "python3-config outputs"    "python3-config --prefix" "/"
echo ""

# ── 8. Rust binary completeness + execution ───────────────
echo "[ 8. Rust binaries ]"
vex use rust@stable > /dev/null 2>&1 || true
# Check symlinks exist
for bin in rustc rustdoc cargo rustfmt cargo-fmt cargo-clippy clippy-driver rust-analyzer rust-gdb rust-lldb; do
    if ls ~/.vex/bin/$bin > /dev/null 2>&1; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
done
# Check executables actually work
check "rustc --version works"          "rustc --version"          "rustc"
check "cargo --version works"          "cargo --version"          "cargo"
check "rustfmt --version works"        "rustfmt --version"        "rustfmt"
check "cargo-clippy --version works"   "cargo-clippy --version"   "clippy"
check "rust-analyzer --version works"  "rust-analyzer --version"  "rust-analyzer"
echo ""

# ── 9. Node binary completeness + execution ───────────────
echo "[ 9. Node binaries ]"
vex use node@20 > /dev/null 2>&1 || true
for bin in node npm npx corepack; do
    if ls ~/.vex/bin/$bin > /dev/null 2>&1; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
done
check "node --version works"  "node --version"  "v"
check "npm --version works"   "npm --version"   "."
check "npx --version works"   "npx --version"   "."
echo ""

# ── 10. Go binary completeness + execution ─────────────────
echo "[ 10. Go binaries ]"
vex use go@latest > /dev/null 2>&1 || true
for bin in go gofmt; do
    if ls ~/.vex/bin/$bin > /dev/null 2>&1; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
done
check "go version works"   "go version"   "go"
check "gofmt -h works"     "gofmt -h 2>&1 | head -1" "usage"
echo ""

# ── 11. Java binary completeness + execution ───────────────
echo "[ 11. Java binaries ]"
vex use java@21 > /dev/null 2>&1 || true
for bin in java javac jar javadoc javap jshell keytool; do
    if ls ~/.vex/bin/$bin > /dev/null 2>&1; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
done
check "java --version works"   "java --version 2>&1 | head -1"  "openjdk"
check "javac --version works"  "javac --version"                "javac"
echo ""

# ── 12. Concurrent install protection ─────────────────────
echo "[ 12. Concurrent install protection ]"
# Create a fake stale lock to simulate conflict
mkdir -p ~/.vex/locks
echo "99999" > ~/.vex/locks/node-18.0.0.lock
check "stale lock shows conflict message" \
    "vex install node@18.0.0" "Another vex process"
rm -f ~/.vex/locks/node-18.0.0.lock
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
