#!/usr/bin/env bash
set -euo pipefail

echo "🍺 Checking Homebrew formula renderer..."

tmp_formula="$(mktemp /tmp/vex-formula.XXXXXX.rb)"
tmp_action="$(mktemp -d /tmp/vex-action.XXXXXX)"
cleanup() {
  rm -f "$tmp_formula"
  rm -rf "$tmp_action"
}
trap cleanup EXIT

bash scripts/render-homebrew-formula.sh \
  "1.2.0" \
  "https://github.com/imnotnoahhh/vex/releases/download/v1.2.0/vex-aarch64-apple-darwin.tar.gz" \
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  "https://github.com/imnotnoahhh/vex/releases/download/v1.2.0/vex-x86_64-apple-darwin.tar.gz" \
  "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
  "$tmp_formula"

grep -q 'class Vex < Formula' "$tmp_formula"
grep -q 'version "1.2.0"' "$tmp_formula"
grep -q 'on_arm do' "$tmp_formula"
grep -q 'on_intel do' "$tmp_formula"
grep -q 'def caveats' "$tmp_formula"
grep -q 'vex init --shell zsh' "$tmp_formula"
grep -q 'assert_match "Would create"' "$tmp_formula"

echo "✅ Homebrew formula renderer passed"
echo ""
echo "🚀 Checking release postflight workflow..."

if command -v ruby >/dev/null 2>&1; then
  ruby -e '
    require "yaml"
    data = YAML.safe_load(File.read(".github/workflows/release-postflight.yml"), permitted_classes: [], aliases: false)
    jobs = data.fetch("jobs").keys.sort
    expected = ["prepare-release", "smoke-release-binary", "update-homebrew-tap", "validate-release-notes"]
    abort("unexpected jobs: #{jobs.inspect}") unless jobs == expected
  '
else
  grep -q '^name: Release Postflight' .github/workflows/release-postflight.yml
  grep -q '^  prepare-release:' .github/workflows/release-postflight.yml
  grep -q '^  validate-release-notes:' .github/workflows/release-postflight.yml
  grep -q '^  smoke-release-binary:' .github/workflows/release-postflight.yml
  grep -q '^  update-homebrew-tap:' .github/workflows/release-postflight.yml
fi

grep -q '^  workflow_call:' .github/workflows/release-postflight.yml
grep -q '^  workflow_dispatch:' .github/workflows/release-postflight.yml
if grep -q '^  release:' .github/workflows/release-postflight.yml; then
  echo "Release postflight workflow should not also listen for release events" >&2
  exit 1
fi
grep -q 'HOMEBREW_TAP_TOKEN' .github/workflows/release-postflight.yml
grep -q 'scripts/render-homebrew-formula.sh' .github/workflows/release-postflight.yml
grep -q 'uses: ./.github/workflows/release-postflight.yml' .github/workflows/release.yml

echo "✅ Release postflight workflow passed"
echo ""
echo "⚙️ Checking setup action tool activation..."

fake_home="$tmp_action/home"
fake_bin="$tmp_action/bin"
fake_log="$tmp_action/vex.log"
mkdir -p \
  "$fake_home/.vex/toolchains/node/20.20.1" \
  "$fake_home/.vex/toolchains/node/20.9.0" \
  "$fake_bin"

cat > "$fake_bin/vex" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$VEX_FAKE_LOG"

case "$1" in
  init)
    mkdir -p "$HOME/.vex"
    ;;
  install)
    ;;
  use)
    if [ "${2:-}" != "node@20.20.1" ]; then
      echo "unexpected activation spec: ${2:-}" >&2
      exit 1
    fi
    ;;
  *)
    echo "unexpected vex command: $*" >&2
    exit 1
    ;;
esac
EOF
chmod +x "$fake_bin/vex"

HOME="$fake_home" PATH="$fake_bin:$PATH" VEX_FAKE_LOG="$fake_log" \
  bash scripts/setup-action-tools.sh --tools "node@20"

grep -q '^install --no-switch node@20$' "$fake_log"
grep -q '^use node@20.20.1$' "$fake_log"

echo "✅ Setup action tool activation passed"
