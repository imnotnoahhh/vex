#!/usr/bin/env bash
set -euo pipefail

REPO_OWNER="imnotnoahhh"
REPO_NAME="vex"
INSTALL_DIR="$HOME/.cargo/bin"
INSTALL_PATH="$INSTALL_DIR/vex"
VERSION=""

usage() {
  cat <<'EOF'
Install vex from GitHub Releases.

Usage:
  install-release.sh [--version <tag>] [--help]

Options:
  --version <tag>  Install a specific release tag (for example: v0.1.0)
  --help           Show this help message
EOF
}

log() {
  printf '%s\n' "$*"
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

OS_NAME="$(uname -s)"
ARCH_NAME="$(uname -m)"

[ "$OS_NAME" = "Darwin" ] || fail "This installer currently supports macOS only"

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

log "Fetching release metadata..."
RELEASE_JSON="$(curl -fsSL "$RELEASE_API_URL")" || fail "Failed to fetch release metadata from GitHub API"

TAG_NAME="$(printf '%s' "$RELEASE_JSON" | tr ',' '\n' | sed -n 's/.*"tag_name" *: *"\([^"]*\)".*/\1/p' | head -n 1)"
[ -n "$TAG_NAME" ] || TAG_NAME="(unknown)"

ASSET_URL="$(
  printf '%s' "$RELEASE_JSON" \
    | tr ',' '\n' \
    | sed -n 's/.*"browser_download_url" *: *"\([^"]*\)".*/\1/p' \
    | sed 's/\\\//\//g' \
    | grep "$TARGET_TRIPLE" \
    | grep -E '\\.tar\\.(gz|xz)$' \
    | head -n 1 \
    || true
)"

if [ -z "$ASSET_URL" ]; then
  fail "No matching macOS asset found for $TARGET_TRIPLE in release $TAG_NAME"
fi

ASSET_NAME="$(basename "$ASSET_URL")"
ARCHIVE_PATH="$TMP_DIR/$ASSET_NAME"
EXTRACT_DIR="$TMP_DIR/extract"

mkdir -p "$EXTRACT_DIR"

log "Downloading $ASSET_NAME..."
curl -fL --retry 3 --retry-delay 1 --output "$ARCHIVE_PATH" "$ASSET_URL" \
  || fail "Failed to download release asset"

log "Extracting archive..."
tar -xf "$ARCHIVE_PATH" -C "$EXTRACT_DIR" || fail "Failed to extract archive"

VEX_BIN="$(find "$EXTRACT_DIR" -type f -name vex | head -n 1)"
[ -n "$VEX_BIN" ] || fail "Could not find vex binary in extracted archive"

mkdir -p "$INSTALL_DIR"
cp "$VEX_BIN" "$INSTALL_PATH"
chmod +x "$INSTALL_PATH"

add_path_line_if_missing() {
  rc_file="$1"
  path_line='export PATH="$HOME/.cargo/bin:$PATH"'

  if [ ! -f "$rc_file" ]; then
    touch "$rc_file"
  fi

  if ! grep -Fqs "$path_line" "$rc_file"; then
    printf '\n%s\n' "$path_line" >> "$rc_file"
    log "Updated $rc_file"
  fi
}

add_path_line_if_missing "$HOME/.zshrc"
add_path_line_if_missing "$HOME/.bashrc"
add_path_line_if_missing "$HOME/.bash_profile"

log "Installed vex $TAG_NAME to $INSTALL_PATH"
log "Run 'vex --version' to verify installation."
log "If your current shell cannot find vex yet, reload config:"
log "  source ~/.zshrc"
log "or"
log "  source ~/.bashrc"
