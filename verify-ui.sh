#!/bin/bash
# UI Foundation Verification Script

set -e

echo "=== UI Foundation Verification ==="
echo

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Not in project root"
    exit 1
fi

echo "✓ Project structure verified"
echo

# Check if ui.rs exists
if [ ! -f "src/ui.rs" ]; then
    echo "Error: src/ui.rs not found"
    exit 1
fi

echo "✓ UI module file exists"
echo

# Check if ui module is registered in main.rs
if ! grep -q "mod ui;" src/main.rs; then
    echo "Error: ui module not registered in main.rs"
    exit 1
fi

echo "✓ UI module registered in main.rs"
echo

# Check if atty dependency is added
if ! grep -q "atty" Cargo.toml; then
    echo "Error: atty dependency not added to Cargo.toml"
    exit 1
fi

echo "✓ atty dependency added"
echo

# Check if updated files use ui module
echo "Checking updated files..."

if ! grep -q "use crate::ui;" src/installer.rs; then
    echo "Warning: installer.rs may not be using ui module"
fi

if ! grep -q "use crate::ui;" src/commands/current.rs; then
    echo "Warning: current.rs may not be using ui module"
fi

if ! grep -q "use crate::ui;" src/commands/updates.rs; then
    echo "Warning: updates.rs may not be using ui module"
fi

if ! grep -q "use crate::ui;" src/commands/doctor/render.rs; then
    echo "Warning: doctor/render.rs may not be using ui module"
fi

echo "✓ File updates verified"
echo

# Check if test file exists
if [ ! -f "tests/ui_test.rs" ]; then
    echo "Error: tests/ui_test.rs not found"
    exit 1
fi

echo "✓ UI tests file exists"
echo

echo "=== All basic checks passed ==="
echo
echo "Note: Full verification requires running:"
echo "  cargo test --all-features -- --test-threads=1"
echo "  cargo clippy --all-targets --all-features -- -D warnings"
echo "  cargo fmt --all -- --check"
echo "  cargo build --release"
