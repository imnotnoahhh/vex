# Homebrew Tap Packaging

vex keeps direct installation as the primary path, but also maintains an optional official Homebrew tap for users who already rely on `brew`.

For CI usage, prefer the repository-root GitHub Action (`uses: imnotnoahhh/vex@v1`) instead of Homebrew. The tap is primarily for interactive local installs.

## Official Tap

The tap repository is intended to live at:

- `imnotnoahhh/homebrew-vex`

Once the tap repository exists and the release automation has a `HOMEBREW_TAP_TOKEN` secret, users can install vex with:

```bash
brew install imnotnoahhh/homebrew-vex/vex
```

## How the Formula Is Generated

The formula is rendered from release artifacts with:

```bash
bash scripts/render-homebrew-formula.sh \
  <semver> \
  https://github.com/imnotnoahhh/vex/releases/download/v<semver>/vex-aarch64-apple-darwin.tar.gz \
  <arm64-sha256> \
  https://github.com/imnotnoahhh/vex/releases/download/v<semver>/vex-x86_64-apple-darwin.tar.gz \
  <x86_64-sha256> \
  /tmp/vex.rb
```

Use the plain semantic version for `<semver>` (for example, `1.7.0` without the leading `v`).

The generated formula:

- installs the prebuilt `vex` binary into Homebrew's `bin`
- keeps `vex` self-contained and independent from Homebrew after installation
- prints `caveats` explaining that shell configuration is still an explicit user step
- smoke-tests `vex --version` and `vex init --dry-run`

## Release Automation

The release postflight workflow:

1. validates that the tagged version has a `CHANGELOG` entry
2. smoke-tests the published macOS release binary
3. downloads the published arm64 and x86_64 archives
4. computes their SHA256 values
5. updates `Formula/vex.rb` in `imnotnoahhh/homebrew-vex`

To enable automatic tap updates, add this repository secret in the main `vex` repository:

- `HOMEBREW_TAP_TOKEN`

The token only needs push access to the tap repository.
