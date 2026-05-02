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

installed_spec_for_request() {
  local spec="$1"
  local tool requested tool_dir

  if [[ "$spec" != *@* ]]; then
    return 1
  fi

  tool="${spec%%@*}"
  requested="${spec#*@}"
  if [ -z "$tool" ] || [ -z "$requested" ]; then
    return 1
  fi

  command -v python3 >/dev/null 2>&1 || return 1
  tool_dir="$HOME/.vex/toolchains/$tool"

  python3 - "$tool_dir" "$requested" "$tool" <<'PY'
from pathlib import Path
import re
import sys

tool_dir = Path(sys.argv[1])
requested = sys.argv[2].strip()
tool = sys.argv[3]


def normalize(version: str) -> str:
    return version[1:] if version.startswith("v") else version


def matches_request(installed: str, request: str) -> bool:
    if request in {"latest", "lts"} or request.startswith("lts-"):
        return True
    installed = normalize(installed)
    request = normalize(request)
    return installed == request or installed.startswith(f"{request}.")


def version_key(version: str) -> list[int]:
    parts = []
    for segment in normalize(version).split("."):
        match = re.match(r"\d+", segment)
        parts.append(int(match.group(0)) if match else -1)
    return parts


if not tool_dir.exists():
    sys.exit(1)

matches = [
    path.name
    for path in tool_dir.iterdir()
    if path.is_dir() and matches_request(path.name, requested)
]

if not matches:
    sys.exit(1)

print(f"{tool}@{max(matches, key=version_key)}")
PY
}

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
  if resolved_spec="$(installed_spec_for_request "$spec")"; then
    printf 'Activating %s...\n' "$resolved_spec"
    vex use "$resolved_spec"
  else
    printf 'Activating %s...\n' "$spec"
    vex use "$spec"
  fi
done
