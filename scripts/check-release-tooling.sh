#!/usr/bin/env bash
set -euo pipefail

echo "🍺 Checking Homebrew formula renderer..."

tmp_formula="$(mktemp /tmp/vex-formula.XXXXXX.rb)"
trap 'rm -f "$tmp_formula"' EXIT

bash scripts/render-homebrew-formula.sh \
  "1.1.1" \
  "https://github.com/imnotnoahhh/vex/releases/download/v1.1.1/vex-aarch64-apple-darwin.tar.gz" \
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  "https://github.com/imnotnoahhh/vex/releases/download/v1.1.1/vex-x86_64-apple-darwin.tar.gz" \
  "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
  "$tmp_formula"

grep -q 'class Vex < Formula' "$tmp_formula"
grep -q 'version "1.1.1"' "$tmp_formula"
grep -q 'on_arm do' "$tmp_formula"
grep -q 'on_intel do' "$tmp_formula"
grep -q 'assert_match "Would create"' "$tmp_formula"

echo "✅ Homebrew formula renderer passed"
echo ""
echo "🚀 Checking release postflight workflow..."

if command -v ruby >/dev/null 2>&1; then
  ruby -e '
    require "yaml"
    data = YAML.load_file(".github/workflows/release-postflight.yml")
    jobs = data.fetch("jobs").keys.sort
    expected = ["smoke-release-binary", "update-homebrew-tap", "validate-release-notes"]
    abort("unexpected jobs: #{jobs.inspect}") unless jobs == expected
  '
else
  grep -q '^name: Release Postflight' .github/workflows/release-postflight.yml
  grep -q '^  validate-release-notes:' .github/workflows/release-postflight.yml
  grep -q '^  smoke-release-binary:' .github/workflows/release-postflight.yml
  grep -q '^  update-homebrew-tap:' .github/workflows/release-postflight.yml
fi

grep -q 'HOMEBREW_TAP_TOKEN' .github/workflows/release-postflight.yml
grep -q 'scripts/render-homebrew-formula.sh' .github/workflows/release-postflight.yml

echo "✅ Release postflight workflow passed"
