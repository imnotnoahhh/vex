#!/bin/bash
# Generate and preview vex documentation

set -e

echo "🔨 Building documentation..."
RUSTDOCFLAGS="--html-in-header docs/header.html" cargo doc --no-deps --quiet

echo "📋 Copying custom CSS..."
cp docs/custom.css target/doc/

echo "✅ Documentation generated successfully!"
echo ""
echo "📚 Documentation location: target/doc/vex/index.html"
echo ""
echo "Key pages:"
echo "  - Main: target/doc/vex/index.html"
echo "  - Error handling: target/doc/vex/error/index.html"
echo "  - Installer: target/doc/vex/installer/index.html"
echo "  - Tools: target/doc/vex/tools/index.html"
echo ""
echo "🌐 Opening in browser..."
open target/doc/vex/index.html
