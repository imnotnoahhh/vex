#!/bin/bash
# vex Shell Hook Test Script
# Tests shell hook generation for zsh, bash, fish, and nushell
# Usage: bash scripts/test-shell-hooks.sh

set -euo pipefail

PASS=0
FAIL=0

pass() { echo "  ✓ $1"; PASS=$((PASS+1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL+1)); }

echo ""
echo "=== vex Shell Hook Tests ==="
echo ""

# ── 1. Zsh Hook ──────────────────────────────────────────────
echo "[ 1. Zsh Hook ]"

ZSH_HOOK=$(vex hook zsh 2>&1)

if echo "$ZSH_HOOK" | grep -q "__vex_use_if_found"; then
    pass "zsh hook contains __vex_use_if_found function"
else
    fail "zsh hook missing __vex_use_if_found function"
fi

if echo "$ZSH_HOOK" | grep -q "__vex_activate_venv"; then
    pass "zsh hook contains __vex_activate_venv function"
else
    fail "zsh hook missing __vex_activate_venv function"
fi

if echo "$ZSH_HOOK" | grep -q "chpwd_functions"; then
    pass "zsh hook registers chpwd_functions"
else
    fail "zsh hook missing chpwd_functions registration"
fi

if echo "$ZSH_HOOK" | grep -q ".tool-versions"; then
    pass "zsh hook checks for .tool-versions"
else
    fail "zsh hook missing .tool-versions check"
fi

if echo "$ZSH_HOOK" | grep -q ".venv"; then
    pass "zsh hook checks for .venv"
else
    fail "zsh hook missing .venv check"
fi

# ── 2. Bash Hook ──────────────────────────────────────────────
echo ""
echo "[ 2. Bash Hook ]"

BASH_HOOK=$(vex hook bash 2>&1)

if echo "$BASH_HOOK" | grep -q "__vex_use_if_found"; then
    pass "bash hook contains __vex_use_if_found function"
else
    fail "bash hook missing __vex_use_if_found function"
fi

if echo "$BASH_HOOK" | grep -q "__vex_activate_venv"; then
    pass "bash hook contains __vex_activate_venv function"
else
    fail "bash hook missing __vex_activate_venv function"
fi

if echo "$BASH_HOOK" | grep -q "PROMPT_COMMAND"; then
    pass "bash hook registers PROMPT_COMMAND"
else
    fail "bash hook missing PROMPT_COMMAND registration"
fi

if echo "$BASH_HOOK" | grep -q ".tool-versions"; then
    pass "bash hook checks for .tool-versions"
else
    fail "bash hook missing .tool-versions check"
fi

if echo "$BASH_HOOK" | grep -q ".venv"; then
    pass "bash hook checks for .venv"
else
    fail "bash hook missing .venv check"
fi

# ── 3. Fish Hook ──────────────────────────────────────────────
echo ""
echo "[ 3. Fish Hook ]"

FISH_HOOK=$(vex hook fish 2>&1)

if echo "$FISH_HOOK" | grep -q "__vex_use_if_found"; then
    pass "fish hook contains __vex_use_if_found function"
else
    fail "fish hook missing __vex_use_if_found function"
fi

if echo "$FISH_HOOK" | grep -q "__vex_activate_venv"; then
    pass "fish hook contains __vex_activate_venv function"
else
    fail "fish hook missing __vex_activate_venv function"
fi

if echo "$FISH_HOOK" | grep -q "fish_prompt"; then
    pass "fish hook registers fish_prompt event"
else
    fail "fish hook missing fish_prompt event"
fi

if echo "$FISH_HOOK" | grep -q ".tool-versions"; then
    pass "fish hook checks for .tool-versions"
else
    fail "fish hook missing .tool-versions check"
fi

if echo "$FISH_HOOK" | grep -q ".venv"; then
    pass "fish hook checks for .venv"
else
    fail "fish hook missing .venv check"
fi

# ── 4. Nushell Hook ──────────────────────────────────────────────
echo ""
echo "[ 4. Nushell Hook ]"

NUSHELL_HOOK=$(vex hook nushell 2>&1)

if echo "$NUSHELL_HOOK" | grep -q "def --env __vex_use_if_found"; then
    pass "nushell hook contains __vex_use_if_found function"
else
    fail "nushell hook missing __vex_use_if_found function"
fi

if echo "$NUSHELL_HOOK" | grep -q "def --env __vex_activate_venv"; then
    pass "nushell hook contains __vex_activate_venv function"
else
    fail "nushell hook missing __vex_activate_venv function"
fi

if echo "$NUSHELL_HOOK" | grep -q "hooks"; then
    pass "nushell hook registers hooks"
else
    fail "nushell hook missing hooks registration"
fi

if echo "$NUSHELL_HOOK" | grep -q ".tool-versions"; then
    pass "nushell hook checks for .tool-versions"
else
    fail "nushell hook missing .tool-versions check"
fi

if echo "$NUSHELL_HOOK" | grep -q ".venv"; then
    pass "nushell hook checks for .venv"
else
    fail "nushell hook missing .venv check"
fi

# ── Summary ──────────────────────────────────────────────
echo ""
echo "=== Summary ==="
echo "  Passed: $PASS"
echo "  Failed: $FAIL"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo "✓ All shell hook tests passed!"
    exit 0
else
    echo "✗ Some shell hook tests failed"
    exit 1
fi
