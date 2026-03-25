#!/usr/bin/env bash
set -euo pipefail

TOOLS=""
AUTO_INSTALL="false"

usage() {
  cat <<'EOF'
Install and activate vex-managed tools for GitHub Actions.

Usage:
  setup-action-tools.sh [--tools "<specs>"] [--auto-install]

Options:
  --tools "<specs>"  Comma, space, or newline separated specs (for example: "node@20 go@1.24")
  --auto-install     Install and activate tools from the current project's version files
EOF
}

fail() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --tools)
      [ "$#" -ge 2 ] || fail "--tools requires a value"
      TOOLS="$2"
      shift 2
      ;;
    --auto-install)
      AUTO_INSTALL="true"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "Unknown argument: $1"
      ;;
  esac
done

[ -n "$TOOLS" ] || [ "$AUTO_INSTALL" = "true" ] || exit 0

if [ -n "$TOOLS" ] && [ "$AUTO_INSTALL" = "true" ]; then
  fail "Use either --tools or --auto-install, not both"
fi

command -v vex >/dev/null 2>&1 || fail "vex must already be installed"

# Ensure ~/.vex directory structure exists (may be absent on fresh or cache-miss runs)
if [ ! -d "$HOME/.vex" ]; then
  vex init --shell skip
fi

if [ "$AUTO_INSTALL" = "true" ]; then
  printf 'Installing tools from project version files...\n'
  vex install --no-switch
  printf 'Activating tools from project version files...\n'
  vex use --auto
  exit 0
fi

NORMALIZED_TOOLS="$(
  printf '%s' "$TOOLS" \
    | tr ',\n\r\t' '    ' \
    | xargs
)"

[ -n "$NORMALIZED_TOOLS" ] || exit 0

read -r -a SPECS <<< "$NORMALIZED_TOOLS"

printf 'Installing requested tools: %s\n' "$NORMALIZED_TOOLS"
vex install --no-switch "${SPECS[@]}"

for spec in "${SPECS[@]}"; do
  printf 'Activating %s...\n' "$spec"
  vex use "$spec"
done
