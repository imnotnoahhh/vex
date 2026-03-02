#!/bin/bash
# Check documentation quality

echo "📝 Checking documentation quality..."
echo ""

# Check for Chinese characters in source files
echo "1. Checking for Chinese characters in doc comments..."
if grep -r "//[!/].*[\u4e00-\u9fff]" src/ 2>/dev/null | head -5; then
    echo "   ⚠️  Found Chinese characters in documentation"
else
    echo "   ✅ No Chinese characters found"
fi
echo ""

# Check if documentation builds without warnings
echo "2. Building documentation..."
if cargo doc --no-deps 2>&1 | grep -i "warning"; then
    echo "   ⚠️  Documentation has warnings"
else
    echo "   ✅ Documentation builds without warnings"
fi
echo ""

# Check if custom CSS exists
echo "3. Checking custom styling..."
if [ -f "docs/custom.css" ] && [ -f "docs/header.html" ]; then
    echo "   ✅ Custom styling files present"
else
    echo "   ⚠️  Custom styling files missing"
fi
echo ""

# Check documentation coverage
echo "4. Checking documentation coverage..."
echo "   Modules with documentation:"
grep -l "^//!" src/*.rs src/tools/*.rs 2>/dev/null | wc -l | xargs echo "   "
echo ""

echo "✅ Documentation quality check complete!"
