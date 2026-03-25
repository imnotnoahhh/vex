#!/bin/bash
# vex v1.2.0 Comprehensive Feature Test Script
# Tests ALL binaries for ALL 5 languages (Node.js, Python, Go, Rust, Java)
# Usage: bash scripts/test-features.sh

set -euo pipefail

# Ensure vex is in PATH (prefer ~/.local/bin, then current directory)
if [ -x "$HOME/.local/bin/vex" ]; then
    export PATH="$HOME/.local/bin:$PATH"
fi
VEX_RELEASE="$(pwd)/target/release"
if [ -x "$VEX_RELEASE/vex" ]; then
    export PATH="$VEX_RELEASE:$PATH"
fi

# Ensure ~/.vex/bin is in PATH (required for testing)
export PATH="$HOME/.vex/bin:$PATH"

# Cleanup on exit
# shellcheck disable=SC2317,SC2329
cleanup() {
    echo ""
    echo "Cleaning up test artifacts..."
    rm -rf "${TMPDIR_TEST:-}" "${PYDIR:-}" 2>/dev/null || true
}
trap cleanup EXIT

PASS=0
FAIL=0

pass() { echo "  ✓ $1"; PASS=$((PASS+1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL+1)); }

check() {
    local desc="$1" cmd="$2" expect="$3"
    local output
    output=$(bash -c "$cmd" 2>&1) || true
    if echo "$output" | grep -q "$expect"; then
        pass "$desc"
    else
        fail "$desc (got: $output)"
    fi
}

# Check with two possible patterns (for locale differences)
check_either() {
    local desc="$1" cmd="$2" expect1="$3" expect2="$4"
    local output
    output=$(bash -c "$cmd" 2>&1) || true
    if echo "$output" | grep -q "$expect1" || echo "$output" | grep -q "$expect2"; then
        pass "$desc"
    else
        fail "$desc (got: $output)"
    fi
}

check_not() {
    local desc="$1" cmd="$2" unexpected="$3"
    local output
    output=$(bash -c "$cmd" 2>&1) || true
    if echo "$output" | grep -q "$unexpected"; then
        fail "$desc (unexpectedly found: $unexpected)"
    else
        pass "$desc"
    fi
}

# Check that symlink points to the correct toolchain version
check_symlink_target() {
    local bin="$1" expected_version="$2"
    if [ ! -L ~/.vex/bin/"$bin" ]; then
        fail "$bin is not a symlink"
        return
    fi
    local target
    target=$(readlink ~/.vex/bin/"$bin")
    if echo "$target" | grep -q "$expected_version"; then
        pass "$bin → toolchain $expected_version"
    else
        fail "$bin points to wrong version (got: $target)"
    fi
}

check_bin_exists() {
    local bin="$1"
    if [ -e ~/.vex/bin/"$bin" ]; then
        pass "$bin symlink exists"
    else
        fail "$bin symlink missing"
    fi
}

# Optional binary: pass if exists, skip (not fail) if missing
# shellcheck disable=SC2317,SC2329
check_bin_exists_optional() {
    local bin="$1"
    if [ -e ~/.vex/bin/"$bin" ]; then
        pass "$bin symlink exists (optional)"
    else
        pass "$bin not present (optional, skipped)"
    fi
}

check_bin_version() {
    local bin="$1" flag="$2" expect="$3"
    local output
    # Use absolute path to ensure we test vex-managed binary, not system binary
    output=$(bash -c "$HOME/.vex/bin/$bin $flag 2>&1 | head -5" 2>&1) || true
    if echo "$output" | grep -qiF -- "$expect"; then
        pass "$bin $flag works"
    else
        fail "$bin $flag failed (got: $output)"
    fi
}

# Optional binary version check: skip if binary not found
# shellcheck disable=SC2317,SC2329
check_bin_version_optional() {
    local bin="$1" flag="$2" expect="$3"
    if [ ! -e ~/.vex/bin/"$bin" ]; then
        pass "$bin not present (optional, skipped)"
        return
    fi
    check_bin_version "$bin" "$flag" "$expect"
}

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  vex v1.2.0 Comprehensive Feature Test Suite              ║"
echo "║  Testing ALL binaries for 5 languages                     ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# ══════════════════════════════════════════════════════════════
# 1. Basic Functionality
# ══════════════════════════════════════════════════════════════
echo "[ 1. Basic Functionality ]"
check "vex --version shows version" "vex --version" "vex"
check_not "vex doctor has no fatal errors" "vex doctor" "Error:"
echo ""

# ══════════════════════════════════════════════════════════════
# 2. Node.js - Complete Binary Test
# ══════════════════════════════════════════════════════════════
echo "[ 2. Node.js v20 - All Binaries ]"
echo "  Installing Node.js 20..."
vex install node@20.11.0 > /dev/null 2>&1 || true
vex use node@20.11.0 > /dev/null 2>&1 || true

# Check all Node.js binaries exist
for bin in node npm npx corepack; do
    check_bin_exists "$bin"
done

# Check symlinks point to correct toolchain version
for bin in node npm npx corepack; do
    check_symlink_target "$bin" "20.11.0"
done

# Test version flags (using absolute paths)
check_bin_version "node" "--version" "v20"
check_bin_version "node" "-v" "v20"
check_bin_version "npm" "--version" "."
check_bin_version "npm" "-v" "."
check_bin_version "npx" "--version" "."
check_bin_version "corepack" "--version" "."

# Test help flags
check_bin_version "node" "--help" "usage"
check_bin_version "npm" "--help" "npm"
check_bin_version "npx" "--help" "npm exec"  # npx shows "npm exec" in help
check_bin_version "corepack" "--help" "corepack"

echo ""

# ══════════════════════════════════════════════════════════════
# 3. Python - Complete Binary Test
# ══════════════════════════════════════════════════════════════
echo "[ 3. Python v3.12 - All Binaries ]"
echo "  Installing Python 3.12..."
vex install python@3.12 > /dev/null 2>&1 || true
vex use python@3.12 > /dev/null 2>&1 || true

# Check all Python binaries exist
for bin in python3 python3.12 pip3 pip3.12 pydoc3 pydoc3.12 2to3 2to3-3.12 python3-config python3.12-config python pip idle3 idle3.12; do
    check_bin_exists "$bin"
done

# Check symlinks point to correct toolchain version (test main binaries)
for bin in python3 python pip3 pip 2to3 pydoc3 python3-config idle3; do
    check_symlink_target "$bin" "3.12"
done

# Test version flags (using absolute paths)
check_bin_version "python3" "--version" "Python 3.12"
check_bin_version "python3" "-V" "Python 3.12"
check_bin_version "python3.12" "--version" "Python 3.12"
check_bin_version "python" "--version" "Python 3.12"
check_bin_version "pip3" "--version" "pip"
check_bin_version "pip3.12" "--version" "pip"
check_bin_version "pip" "--version" "pip"
check_bin_version "2to3" "--version" "2to3"
check_bin_version "2to3-3.12" "--version" "2to3"

# Test help flags
check_bin_version "python3" "--help" "usage"
check_bin_version "python3" "-h" "usage"
check_bin_version "pip3" "--help" "Usage"
check_bin_version "pip3" "-h" "Usage"
check_bin_version "2to3" "--help" "Usage"
check_bin_version "2to3" "-h" "Usage"
check_bin_version "pydoc3" "-h" "pydoc"
check_bin_version "python3-config" "--help" "Usage"
check_bin_version "python3-config" "--prefix" "/"
check_bin_version "python3-config" "--cflags" "-I"

echo ""

# ══════════════════════════════════════════════════════════════
# 4. Python Virtual Environment (Critical)
# ══════════════════════════════════════════════════════════════
echo "[ 4. Python Virtual Environment - Critical Tests ]"
PYDIR=$(mktemp -d)

# Test error messages
check "python freeze without .venv shows error" \
    "(cd $PYDIR && vex python freeze 2>&1)" "No .venv found"
check "python sync without lock shows error" \
    "(cd $PYDIR && vex python sync 2>&1)" "No requirements.lock found"

# Test venv creation
echo "  Creating virtual environment..."
(cd "$PYDIR" && vex python init > /dev/null 2>&1)
if [ -d "$PYDIR/.venv" ]; then
    pass "python init creates .venv directory"
else
    fail "python init failed to create .venv"
fi

# Verify venv structure
if [ -f "$PYDIR/.venv/bin/python" ]; then
    pass ".venv/bin/python exists"
else
    fail ".venv/bin/python missing"
fi

if [ -f "$PYDIR/.venv/bin/pip" ]; then
    pass ".venv/bin/pip exists"
else
    fail ".venv/bin/pip missing"
fi

# Test freeze
echo "  Testing freeze..."
(cd "$PYDIR" && vex python freeze > /dev/null 2>&1)
if [ -f "$PYDIR/requirements.lock" ]; then
    pass "python freeze creates requirements.lock"
else
    fail "python freeze failed to create requirements.lock"
fi

# Test sync (restore from lock)
echo "  Testing sync..."
rm -rf "$PYDIR/.venv"
(cd "$PYDIR" && vex python sync > /dev/null 2>&1)
if [ -d "$PYDIR/.venv" ]; then
    pass "python sync restores .venv from lock"
else
    fail "python sync failed to restore .venv"
fi

# Test venv activation
if [ -f "$PYDIR/.venv/bin/activate" ]; then
    pass ".venv/bin/activate script exists"
else
    fail ".venv/bin/activate script missing"
fi

rm -rf "$PYDIR"
echo ""

# ══════════════════════════════════════════════════════════════
# 5. Go - Complete Binary Test
# ══════════════════════════════════════════════════════════════
echo "[ 5. Go (latest) - All Binaries ]"
echo "  Installing Go (latest stable)..."

# Try to install Go with retries
GO_INSTALLED=false
for _attempt in 1 2 3; do
    echo "  Attempt $_attempt/3..."
    if vex install go@latest > /dev/null 2>&1; then
        GO_INSTALLED=true
        break
    fi
    if [ $_attempt -lt 3 ]; then
        echo "  Retrying in 3 seconds..."
        sleep 3
    fi
done

if [ "$GO_INSTALLED" = "false" ]; then
    fail "Go installation failed after 3 attempts (check network/proxy)"
    echo ""
else
    vex use go@latest > /dev/null 2>&1 || true

    # Check all Go binaries exist
    for bin in go gofmt; do
        check_bin_exists "$bin"
    done

    # Check symlinks point to correct toolchain
    for bin in go gofmt; do
        check_symlink_target "$bin" "go"
    done

    # Test version flags (Go uses different syntax, using absolute paths)
    check_bin_version "go" "version" "go version"
    check_bin_version "gofmt" "-h" "usage"

    # Test help flags
    check_bin_version "go" "help" "Go is a tool"

    echo ""
fi

# ══════════════════════════════════════════════════════════════
# 6. Rust - Complete Binary Test
# ══════════════════════════════════════════════════════════════
echo "[ 6. Rust - All Binaries ]"
echo "  Installing Rust (latest stable)..."
vex install rust@stable > /dev/null 2>&1 || true
vex use rust@stable > /dev/null 2>&1 || true

# Check all Rust binaries exist (including rust-gdbgui)
for bin in rustc cargo rustdoc rustfmt cargo-fmt clippy-driver cargo-clippy rust-gdb rust-gdbgui rust-lldb rust-analyzer; do
    check_bin_exists "$bin"
done

# Check symlinks point to correct toolchain
for bin in rustc cargo rustfmt cargo-clippy rust-analyzer; do
    check_symlink_target "$bin" "rust"
done

# Test version flags (using absolute paths)
check_bin_version "rustc" "--version" "rustc"
check_bin_version "rustc" "-V" "rustc"
check_bin_version "cargo" "--version" "cargo"
check_bin_version "cargo" "-V" "cargo"
check_bin_version "rustdoc" "--version" "rustdoc"
check_bin_version "rustfmt" "--version" "rustfmt"
check_bin_version "cargo-fmt" "--version" "rustfmt"  # cargo-fmt shows rustfmt version
check_bin_version "clippy-driver" "--version" "clippy"
check_bin_version "cargo-clippy" "--version" "clippy"
check_bin_version "rust-analyzer" "--version" "rust-analyzer"

# Test help flags
check_bin_version "rustc" "--help" "Usage"
check_bin_version "rustc" "-h" "Usage"
check_bin_version "cargo" "--help" "Rust's package manager"
check_bin_version "cargo" "-h" "Rust's package manager"
check_bin_version "rustfmt" "--help" "Format Rust code"
check_bin_version "cargo-clippy" "--help" "Checks a package"

echo ""

# ══════════════════════════════════════════════════════════════
# 7. Java - Complete Binary Test
# ══════════════════════════════════════════════════════════════
echo "[ 7. Java JDK 21 - All Binaries ]"
echo "  Installing Java 21..."
vex install java@21 > /dev/null 2>&1 || true
vex use java@21 > /dev/null 2>&1 || true

# Check all Java binaries exist (all 29 JDK 21 tools for macOS)
for bin in java javac jar javadoc javap jshell keytool jarsigner jdb jdeps jfr jhsdb jinfo jmap jps jstack jstat serialver jrunscript jcmd jconsole jdeprscan jimage jlink jmod jpackage jstatd jwebserver rmiregistry; do
    check_bin_exists "$bin"
done

# Check symlinks point to correct toolchain (test main binaries)
for bin in java javac jar javadoc javap jshell keytool jcmd jdeps jlink jpackage jimage; do
    check_symlink_target "$bin" "21"
done

# Test version flags (Java uses -version not --version, using absolute paths)
check_bin_version "java" "-version" "openjdk"
check_bin_version "javac" "-version" "javac"
check_bin_version "jar" "--version" "jar"
check_bin_version "javadoc" "--version" "javadoc"
check_bin_version "javap" "-version" "21"
check_bin_version "jshell" "--version" "jshell"
check_either "keytool -help works" "keytool -help 2>&1 | head -5" "密钥" "Key and Certificate"
check_bin_version "jarsigner" "-help" "jarsigner"
check_bin_version "jdb" "-version" "jdb"
check_bin_version "jdeps" "--version" "21"
check_bin_version "jfr" "--version" "21"
check_bin_version "jps" "-version" "jps"
check_bin_version "jstack" "-version" "jstack"
check_bin_version "jstat" "-version" "jstat"
check_bin_version "jlink" "--version" "21"
check_bin_version "jmod" "--version" "21"
check_bin_version "jpackage" "--version" "21"
check_bin_version "jwebserver" "--version" "21"
check_bin_version "jimage" "--version" "21"
check_bin_version "jdeprscan" "--version" "21"

# Test help flags (support both Chinese and English locale)
check_either "java -help works" "java -help 2>&1 | head -5" "用法" "Usage"
check_either "javac -help works" "javac -help 2>&1 | head -5" "用法" "Usage"
check_either "jar --help works" "jar --help 2>&1 | head -5" "用法" "Usage"
check_either "javadoc --help works" "javadoc --help 2>&1 | head -5" "用法" "Usage"

echo ""

# ══════════════════════════════════════════════════════════════
# 8. Cross-Language Integration Tests
# ══════════════════════════════════════════════════════════════
echo "[ 8. Cross-Language Integration ]"

# Test switching between versions
check "switch to node@20.11.0" "vex use node@20.11.0" "Switched"
check "switch to python@3.12" "vex use python@3.12" "Switched"
if [ "$GO_INSTALLED" = "true" ]; then
    check "switch to go@latest" "vex use go@latest" "Switched"
fi
check "switch to rust@stable" "vex use rust@stable" "Switched"
check "switch to java@21" "vex use java@21" "Switched"

# Test current command
check "current shows active versions" "vex current" "node"
check "current shows active versions" "vex current" "python"
if [ "$GO_INSTALLED" = "true" ]; then
    check "current shows active versions" "vex current" "go"
fi
check "current shows active versions" "vex current" "rust"
check "current shows active versions" "vex current" "java"

# Test global defaults
vex global node@20.11.0 > /dev/null 2>&1
vex global python@3.12 > /dev/null 2>&1
check "global shows node" "cat ~/.vex/tool-versions" "node"
check "global shows python" "cat ~/.vex/tool-versions" "python"

echo ""

# ══════════════════════════════════════════════════════════════
# 9. Version Source Detection
# ══════════════════════════════════════════════════════════════
echo "[ 9. Version Source Detection ]"
TMPDIR_TEST=$(mktemp -d)

# Test global default
check "global version shows 'Global default'" \
    "(cd $TMPDIR_TEST && vex current)" "Global default"

# Test project override
echo "node 20.11.0" > "$TMPDIR_TEST/.tool-versions"
check "project .tool-versions shows 'Project override'" \
    "(cd $TMPDIR_TEST && vex current)" "Project override"

rm -rf "$TMPDIR_TEST"
echo ""

# ══════════════════════════════════════════════════════════════
# 10. List Remote Filters
# ══════════════════════════════════════════════════════════════
echo "[ 10. List Remote Filters ]"
check "--filter latest returns 1 version" \
    "vex list-remote node --filter latest" "1 versions"
check "--filter lts returns LTS versions" \
    "vex list-remote node --filter lts" "LTS:"
check "--filter major sorted descending" \
    "vex list-remote node --filter major" "25.8.1"
echo ""

# ══════════════════════════════════════════════════════════════
# 11. Install Options
# ══════════════════════════════════════════════════════════════
echo "[ 11. Install Options ]"
check "install switches by default" \
    "vex install node@20.10.0" "Switched"
check "install --no-switch does not switch" \
    "vex install node@20.9.0 --no-switch" "Installed:"
echo ""

# ══════════════════════════════════════════════════════════════
# 12. Dynamic Binary Detection
# ══════════════════════════════════════════════════════════════
echo "[ 12. Dynamic Binary Detection ]"

# Java jnativescan (only in Java 25+)
vex use java@21 > /dev/null 2>&1 || true
if ls ~/.vex/bin/jnativescan > /dev/null 2>&1; then
    fail "Java 21 should NOT have jnativescan"
else
    pass "Java 21 does not have jnativescan"
fi

# Node corepack (removed in Node 25+)
vex use node@20.11.0 > /dev/null 2>&1 || true
if [ -e ~/.vex/bin/corepack ]; then
    pass "Node 20 has corepack"
else
    fail "Node 20 should have corepack"
fi

echo ""

# ══════════════════════════════════════════════════════════════
# 13. Concurrent Install Protection
# ══════════════════════════════════════════════════════════════
echo "[ 13. Concurrent Install Protection ]"
mkdir -p ~/.vex/locks
LOCK_VER="20.8.0"
LOCK_FILE="$HOME/.vex/locks/node-${LOCK_VER}.lock"

# Create a lock using Python
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

check "lock conflict shows error message" \
    "vex install node@${LOCK_VER}" "Another vex process"

kill $LOCK_PID 2>/dev/null || true
wait $LOCK_PID 2>/dev/null || true
rm -f "$LOCK_FILE"
echo ""

# ══════════════════════════════════════════════════════════════
# 14. Missing Commands Coverage
# ══════════════════════════════════════════════════════════════
echo "[ 14. Missing Commands Coverage ]"

# Test init command
INIT_TEST_DIR=$(mktemp -d)
check "init creates vex directory structure" \
    "HOME=$INIT_TEST_DIR vex init 2>&1" "Created"
if [ -d "$INIT_TEST_DIR/.vex/bin" ] && [ -d "$INIT_TEST_DIR/.vex/toolchains" ]; then
    pass "init creates bin and toolchains directories"
else
    fail "init failed to create directory structure"
fi
rm -rf "$INIT_TEST_DIR"

# Test list command
check "list node shows installed versions" "vex list node" "20.11.0"
check "list python shows installed versions" "vex list python" "3.12"
check "list shows node versions" "vex list node" "node"

# Test uninstall command (install a temp version first)
vex install node@20.8.0 --no-switch > /dev/null 2>&1 || true
check "uninstall removes version" "vex uninstall node@20.8.0" "Uninstalled"
check_not "uninstalled version not in list" "vex list node" "20.8.0"

# Test env command
check "env outputs shell hook" "vex env zsh" "__vex_use_if_found"
check "env outputs activation hook" "vex env zsh" "__vex_activate_venv"

# Test local command
LOCAL_TEST_DIR=$(mktemp -d)
(cd "$LOCAL_TEST_DIR" && vex local node@20.11.0 > /dev/null 2>&1)
if [ -f "$LOCAL_TEST_DIR/.tool-versions" ]; then
    pass "local creates .tool-versions"
    check "local pins version" "cat \"$LOCAL_TEST_DIR/.tool-versions\"" "node 20.11.0"
else
    fail "local failed to create .tool-versions"
fi
rm -rf "$LOCAL_TEST_DIR"

# Test upgrade command
check "upgrade installs latest version" "vex upgrade node 2>&1" "Switched"

# Test alias command help surface
check "alias help shows set subcommand" "vex alias --help" "set"
check "alias help shows list subcommand" "vex alias --help" "list"
check "alias help shows delete subcommand" "vex alias --help" "delete"
check "alias help shows command usage" "vex alias --help" "Usage: vex alias <COMMAND>"

# Test self-update command (dry run check)
check "self-update checks for updates" "vex self-update --help" "Update vex"

# Test help command
check "help shows usage" "vex help" "Usage"
check "help install shows install help" "vex help install" "Install a tool"

echo ""

# ══════════════════════════════════════════════════════════════
# 15. Doctor Health Check
# ══════════════════════════════════════════════════════════════
echo "[ 15. Doctor Health Check ]"
check "doctor checks vex directory" "vex doctor" "Checking vex directory"
check "doctor checks directory structure" "vex doctor" "Checking directory structure"
check "doctor checks installed tools" "vex doctor" "Checking installed tools"
check "doctor checks symlinks integrity" "vex doctor" "Checking symlinks integrity"
check "doctor checks binary executability" "vex doctor" "Checking binary executability"
check "doctor checks network connectivity" "vex doctor" "Checking network connectivity"
echo ""

# ══════════════════════════════════════════════════════════════
# 16. Focused Management Workflows
# ══════════════════════════════════════════════════════════════
echo "[ 16. Focused Management Workflows ]"
if VEX_BIN="$(command -v vex)" bash "$(pwd)/scripts/test-management-features.sh"; then
    pass "management workflow bash suite passes"
else
    fail "management workflow bash suite failed"
fi
echo ""

# ══════════════════════════════════════════════════════════════
# 17. Shell Hook Workflows
# ══════════════════════════════════════════════════════════════
echo "[ 17. Shell Hook Workflows ]"
if VEX_BIN="$(command -v vex)" bash "$(pwd)/scripts/test-shell-hooks.sh"; then
    pass "shell hook bash suite passes"
else
    fail "shell hook bash suite failed"
fi
echo ""

# ══════════════════════════════════════════════════════════════
# Summary
# ══════════════════════════════════════════════════════════════
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                    Test Summary                            ║"
echo "╠════════════════════════════════════════════════════════════╣"
printf "║  %-20s %37s ║\n" "Passed:" "$PASS"
printf "║  %-20s %37s ║\n" "Failed:" "$FAIL"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo "✅ All tests passed!"
    exit 0
else
    echo "❌ Some tests failed. Check output above."
    exit 1
fi
