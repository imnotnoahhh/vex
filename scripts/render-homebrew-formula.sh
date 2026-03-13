#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 6 ]]; then
  echo "usage: $0 <version> <arm-url> <arm-sha256> <intel-url> <intel-sha256> <output-path>" >&2
  exit 1
fi

version="$1"
arm_url="$2"
arm_sha="$3"
intel_url="$4"
intel_sha="$5"
output_path="$6"

mkdir -p "$(dirname "$output_path")"

cat >"$output_path" <<EOF
class Vex < Formula
  desc "A fast, multi-language version manager for macOS"
  homepage "https://github.com/imnotnoahhh/vex"
  version "$version"
  license "MIT"

  on_arm do
    url "$arm_url"
    sha256 "$arm_sha"
  end

  on_intel do
    url "$intel_url"
    sha256 "$intel_sha"
  end

  def install
    bin.install "vex"
  end

  def caveats
    <<~EOS
      Homebrew installs the vex binary, but does not modify your shell configuration.

      Preview the initialization steps:
        vex init --dry-run

      Configure zsh integration:
        vex init --shell zsh

      Or print shell hooks manually:
        vex env zsh
    EOS
  end

  test do
    assert_match "vex #{version}", shell_output("#{bin}/vex --version")
    assert_match "Would create", shell_output("#{bin}/vex init --dry-run")
  end
end
EOF
