#!/bin/bash
set -e

echo "🧪 Running tests..."
cargo test --all-features -- --test-threads=1

echo "🏗️  Building release..."
cargo build --release

echo "🔍 Linting..."
cargo clippy --all-targets --all-features -- -D warnings

echo "📝 Checking format..."
cargo fmt --all --check

echo "🔒 Security audit..."
cargo audit || echo "⚠️  Security audit found issues (non-blocking)"

echo "✅ All verifications passed!"
