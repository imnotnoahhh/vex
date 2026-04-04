#!/bin/bash
# Live network smoke test for official Rust targets/components managed by vex.

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

PASS=0
FAIL=0
TEMP_HOME_CREATED=0

pass() {
    echo "  ✓ $1"
    PASS=$((PASS + 1))
}

fail() {
    echo "  ✗ $1"
    FAIL=$((FAIL + 1))
}

cleanup() {
    if [ "$TEMP_HOME_CREATED" -eq 1 ]; then
        rm -rf "$HOME"
    fi
}
trap cleanup EXIT

if [ -n "${VEX_TEST_HOME:-}" ]; then
    export HOME="$VEX_TEST_HOME"
    mkdir -p "$HOME"
else
    HOME="$(mktemp -d "${TMPDIR:-/tmp}/vex-rust-live.XXXXXX")"
    export HOME
    TEMP_HOME_CREATED=1
fi

VEX_BIN_DIR="$(dirname "$VEX_BIN")"
export PATH="$ROOT_DIR/target/debug:$VEX_BIN_DIR:$PATH"

echo ""
echo "============================================================"
echo "vex Rust extension live smoke"
echo "Official Rust install + iOS targets + rust-src component"
echo "============================================================"

"$VEX_BIN" init --shell skip >/dev/null
"$VEX_BIN" install rust@stable --no-switch
"$VEX_BIN" use rust@stable

RUST_VERSION="$("$VEX_BIN" current --json | python3 -c 'import json,sys; data=json.load(sys.stdin); rust=[t for t in data["tools"] if t["tool"]=="rust"]; print(rust[0]["version"] if rust else "")')"
if [ -z "$RUST_VERSION" ]; then
    echo "Unable to resolve installed Rust version" >&2
    exit 1
fi
TOOLCHAIN_DIR="$HOME/.vex/toolchains/rust/$RUST_VERSION"
METADATA_FILE="$TOOLCHAIN_DIR/.vex-metadata.json"

"$VEX_BIN" rust target add aarch64-apple-ios aarch64-apple-ios-sim
"$VEX_BIN" rust component add rust-src

TARGET_LIST="$("$VEX_BIN" rust target list)"
COMPONENT_LIST="$("$VEX_BIN" rust component list)"

if [[ "$TARGET_LIST" == *"aarch64-apple-ios"* && "$TARGET_LIST" == *"aarch64-apple-ios-sim"* ]]; then
    pass "rust target list shows installed iOS targets"
else
    fail "rust target list did not show both installed iOS targets"
fi

if [[ "$COMPONENT_LIST" == *"rust-src"* ]]; then
    pass "rust component list shows rust-src"
else
    fail "rust component list did not show rust-src"
fi

for path in \
    "$TOOLCHAIN_DIR/rust-std-aarch64-apple-ios" \
    "$TOOLCHAIN_DIR/rust-std-aarch64-apple-ios-sim" \
    "$TOOLCHAIN_DIR/rust-src"
do
    if [ -e "$path" ]; then
        pass "$(basename "$path") extracted into the toolchain"
    else
        fail "missing expected extracted component: $path"
    fi
done

for link_path in \
    "$TOOLCHAIN_DIR/rustc/lib/rustlib/aarch64-apple-ios/lib" \
    "$TOOLCHAIN_DIR/rustc/lib/rustlib/aarch64-apple-ios-sim/lib" \
    "$TOOLCHAIN_DIR/rustc/lib/rustlib/src"
do
    if [ -L "$link_path" ]; then
        pass "$(basename "$(dirname "$link_path")") link exists in rustc sysroot"
    else
        fail "missing expected sysroot link: $link_path"
    fi
done

if [ -f "$METADATA_FILE" ] \
    && grep -Fq '"name": "aarch64-apple-ios"' "$METADATA_FILE" \
    && grep -Fq '"name": "aarch64-apple-ios-sim"' "$METADATA_FILE" \
    && grep -Fq '"name": "rust-src"' "$METADATA_FILE"; then
    pass "toolchain metadata records Rust extensions"
else
    fail "toolchain metadata did not record the installed Rust extensions"
fi

"$VEX_BIN" rust target remove aarch64-apple-ios-sim
"$VEX_BIN" rust component remove rust-src

if [ ! -e "$TOOLCHAIN_DIR/rust-std-aarch64-apple-ios-sim" ] \
    && [ ! -e "$TOOLCHAIN_DIR/rustc/lib/rustlib/aarch64-apple-ios-sim/lib" ]; then
    pass "removing a managed Rust target cleans extracted files and sysroot link"
else
    fail "managed Rust target removal left files behind"
fi

if [ ! -e "$TOOLCHAIN_DIR/rust-src" ] \
    && [ ! -e "$TOOLCHAIN_DIR/rustc/lib/rustlib/src" ]; then
    pass "removing rust-src cleans extracted files and sysroot link"
else
    fail "rust-src removal left files behind"
fi

if [ -f "$METADATA_FILE" ] \
    && ! grep -Fq '"name": "aarch64-apple-ios-sim"' "$METADATA_FILE" \
    && ! grep -Fq '"name": "rust-src"' "$METADATA_FILE" \
    && grep -Fq '"name": "aarch64-apple-ios"' "$METADATA_FILE"; then
    pass "metadata updates after Rust extension removal"
else
    fail "metadata did not update after Rust extension removal"
fi

echo ""
echo "============================================================"
echo "Passed : $PASS"
echo "Failed : $FAIL"
echo "Rust   : $RUST_VERSION"
echo "HOME   : $HOME"
echo "============================================================"

if [ "$FAIL" -ne 0 ]; then
    exit 1
fi
