#!/bin/bash
# Check documentation quality

set -euo pipefail

echo "📝 Checking documentation quality..."
echo ""

markdown_files() {
    find . -type f -name "*.md" \
        -not -path "./target/*" \
        -not -path "./.git/*" \
        | sort
}

check_markdown_links() {
    local broken=0
    while IFS= read -r file; do
        local dir
        dir=$(dirname "$file")
        while IFS= read -r target; do
            target="${target%%#*}"
            case "$target" in
                ""|\#*|http://*|https://*|mailto:*)
                    continue
                    ;;
            esac
            if [[ "$target" == /* ]]; then
                continue
            fi
            if [ ! -e "$dir/$target" ]; then
                echo "   ❌ $file -> $target"
                broken=$((broken + 1))
            fi
        done < <(
            perl -ne 'while (/\[[^]]+\]\(([^)]+)\)/g) { print "$1\n" }' "$file" \
                | sort -u
        )
    done < <(markdown_files)

    if [ "$broken" -eq 0 ]; then
        echo "   ✅ Markdown relative links look valid"
    else
        echo "   ⚠️  Found $broken broken Markdown links"
    fi
}

echo "1. Checking for Chinese characters in Rust doc comments..."
if grep -r "//[!/].*[\u4e00-\u9fff]" src/ 2>/dev/null | head -5; then
    echo "   ⚠️  Found Chinese characters in documentation comments"
else
    echo "   ✅ No Chinese characters found"
fi
echo ""

echo "2. Building Rust documentation..."
if cargo doc --no-deps 2>&1 | grep -i "warning"; then
    echo "   ⚠️  Documentation has warnings"
else
    echo "   ✅ Documentation builds without warnings"
fi
echo ""

echo "3. Checking custom styling..."
if [ -f "docs/custom.css" ] && [ -f "docs/header.html" ]; then
    echo "   ✅ Custom styling files present"
else
    echo "   ⚠️  Custom styling files missing"
fi
echo ""

echo "4. Checking documentation coverage..."
echo "   Rust modules with crate/module docs:"
grep -l "^//!" src/*.rs src/tools/*.rs 2>/dev/null | wc -l | xargs echo "   "
echo ""

echo "5. Checking Markdown inventory..."
markdown_files | wc -l | xargs echo "   Markdown files:"
echo ""

echo "6. Checking Markdown relative links..."
check_markdown_links
echo ""

echo "✅ Documentation quality check complete!"
