#!/usr/bin/env bash
set -euo pipefail

REPO_OWNER="imnotnoahhh"
REPO_NAME="vex"
INSTALL_DIR="${HOME}/.local/bin"
INSTALL_PATH="${INSTALL_DIR}/vex"
VERSION=""

usage() {
  cat <<'EOF'
Install vex from GitHub Releases for CI usage.

Usage:
  install-ci-release.sh [--version <tag>] [--help]

Options:
  --version <tag>  Install a specific release tag (for example: v1.2.0)
  --help           Show this help message
EOF
}

fail() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      [ "$#" -ge 2 ] || fail "--version requires a value"
      VERSION="$2"
      shift 2
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

command -v curl >/dev/null 2>&1 || fail "curl is required"
command -v tar >/dev/null 2>&1 || fail "tar is required"
command -v shasum >/dev/null 2>&1 || fail "shasum is required"
command -v python3 >/dev/null 2>&1 || fail "python3 is required"

if [ -n "$VERSION" ]; then
  if ! printf '%s' "$VERSION" | grep -qE '^v[0-9]+\.[0-9]+\.[0-9]+([-+].+)?$'; then
    fail "Invalid version tag format: '$VERSION'. Expected pattern: v1.2.3"
  fi
fi

OS_NAME="$(uname -s)"
ARCH_NAME="$(uname -m)"

[ "$OS_NAME" = "Darwin" ] || fail "This installer currently supports macOS runners only"

case "$ARCH_NAME" in
  arm64)
    TARGET_TRIPLE="aarch64-apple-darwin"
    ;;
  x86_64)
    TARGET_TRIPLE="x86_64-apple-darwin"
    ;;
  *)
    fail "Unsupported architecture: $ARCH_NAME"
    ;;
esac

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

if [ -n "$VERSION" ]; then
  RELEASE_API_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/tags/$VERSION"
else
  RELEASE_API_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
fi

printf 'Fetching vex release metadata...\n'
if [ -n "${GITHUB_TOKEN:-}" ]; then
  RELEASE_JSON="$(
    curl -fsSL \
      -H "Accept: application/vnd.github+json" \
      -H "X-GitHub-Api-Version: 2022-11-28" \
      -H "Authorization: Bearer ${GITHUB_TOKEN}" \
      "$RELEASE_API_URL"
  )" || fail "Failed to fetch release metadata"
else
  RELEASE_JSON="$(
    curl -fsSL \
      -H "Accept: application/vnd.github+json" \
      -H "X-GitHub-Api-Version: 2022-11-28" \
      "$RELEASE_API_URL"
  )" || fail "Failed to fetch release metadata"
fi
RELEASE_JSON_PATH="${TMP_DIR}/release.json"
printf '%s' "$RELEASE_JSON" > "$RELEASE_JSON_PATH"

PARSED_RELEASE="$(
  python3 - "$RELEASE_JSON_PATH" "$TARGET_TRIPLE" <<'PY'
import json
import sys
from pathlib import Path

release = json.loads(Path(sys.argv[1]).read_text())
target = sys.argv[2]

tag_name = release.get("tag_name", "")
asset_url = ""

for asset in release.get("assets", []):
    url = asset.get("browser_download_url", "")
    if target in url and (url.endswith(".tar.gz") or url.endswith(".tar.xz")):
        asset_url = url
        break

print(tag_name)
print(asset_url)
PY
)"

TAG_NAME="$(printf '%s\n' "$PARSED_RELEASE" | sed -n '1p')"
[ -n "$TAG_NAME" ] || TAG_NAME="(unknown)"

ASSET_URL="$(printf '%s\n' "$PARSED_RELEASE" | sed -n '2p')"

[ -n "$ASSET_URL" ] || fail "No matching macOS asset found for $TARGET_TRIPLE in release $TAG_NAME"

ASSET_NAME="$(basename "$ASSET_URL")"
ARCHIVE_PATH="${TMP_DIR}/${ASSET_NAME}"
EXTRACT_DIR="${TMP_DIR}/extract"
CHECKSUM_PATH="${TMP_DIR}/checksum.txt"

mkdir -p "$EXTRACT_DIR"

printf 'Downloading %s...\n' "$ASSET_NAME"
curl -fL --retry 3 --retry-delay 1 --output "$ARCHIVE_PATH" "$ASSET_URL" \
  || fail "Failed to download release asset"

CHECKSUM_URL="${ASSET_URL}.sha256"
if curl -fsSL "$CHECKSUM_URL" -o "$CHECKSUM_PATH" 2>/dev/null; then
  EXPECTED="$(awk '{print $1}' "$CHECKSUM_PATH")"
  ACTUAL="$(shasum -a 256 "$ARCHIVE_PATH" | awk '{print $1}')"
  [ "$EXPECTED" = "$ACTUAL" ] || fail "Checksum verification failed for $ASSET_NAME"
  printf 'Verified checksum for %s.\n' "$ASSET_NAME"
else
  printf 'Checksum file not found for %s, skipping verification.\n' "$ASSET_NAME"
fi

printf 'Extracting vex archive...\n'
tar -xf "$ARCHIVE_PATH" -C "$EXTRACT_DIR" || fail "Failed to extract release archive"

VEX_BIN="$(find "$EXTRACT_DIR" -type f -name vex | head -n 1)"
[ -n "$VEX_BIN" ] || fail "Could not find vex binary in extracted archive"

mkdir -p "$INSTALL_DIR"
cp "$VEX_BIN" "$INSTALL_PATH"
chmod +x "$INSTALL_PATH"

printf 'Installed vex %s to %s\n' "$TAG_NAME" "$INSTALL_PATH"
