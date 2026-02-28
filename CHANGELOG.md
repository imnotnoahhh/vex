# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4] - 2026-03-01

### Fixed

- **Uninstall symlink cleanup** — `vex uninstall` now removes stale `current/` and `bin/` symlinks when uninstalling the active version, preventing dangling links
- **Go verify hint** — `vex use go@x` now prints `go version` instead of incorrect `go --version`
- **Rust complete toolchain** — Rust installation now includes all components (rustc, rustdoc, cargo, rustfmt, cargo-fmt, cargo-clippy, clippy-driver, rust-analyzer, rust-gdb, rust-gdbgui, rust-lldb) with proper `post_install` hook that links rust-std to sysroot and shared libraries for clippy/rustfmt/rust-analyzer
- **Java complete binaries** — Expanded Java `bin_names()` from 3 to all 30 JDK executables shipped by Eclipse Temurin
- **Rust missing binaries** — Added rustdoc, clippy-driver, rust-gdb, rust-gdbgui, rust-lldb to Rust `bin_names()` and `bin_paths()`

### Added

- **`post_install` hook** — Tool trait now supports a `post_install()` method for tool-specific setup after extraction (used by Rust for sysroot and library linking)

## [0.1.3] - 2026-03-01

### Added

- **Shell integration prompt** — Install script now asks whether to configure shell hook (`eval "$(vex env ...)"`) after installation, eliminating the need for a separate `vex init` step

## [0.1.2] - 2026-02-28

### Fixed

- **Node.js corepack support** — Added `corepack` to `bin_names()` so vex now creates symlinks for the corepack binary shipped with Node.js v16+, enabling `corepack enable pnpm/yarn`
- **Install script JSON parsing** — Fixed sed patterns to handle spaces in GitHub API JSON responses (`"key": "value"` instead of `"key":"value"`)
- **Install script archive format** — Updated grep pattern to match both `.tar.gz` and `.tar.xz` files (cargo-dist produces `.tar.xz`)
- **Install script silent exit** — Added `|| true` to grep pipeline to prevent `pipefail` from causing silent script termination when no assets match

### Changed

- **Install path** — Changed default installation directory from `~/.cargo/bin` to `~/.local/bin` following XDG standard, avoiding semantic confusion with Rust toolchain
- **Shell detection** — Install script now detects current shell (`$SHELL`) and only updates the appropriate rc file (`.zshrc` for zsh, `.bash_profile`/`.bashrc` for bash) instead of modifying all shell configs

## [0.1.1] - 2026-02-28

### Added

- **Version aliases** — `latest`, `lts`, `lts-<codename>` (Node), `stable` (Rust), minor version matching (Go `1.23` → latest `1.23.x`)
- **`vex upgrade <tool>`** — one-command upgrade to the latest version
- **`vex alias <tool>`** — show all available aliases and their resolved values
- **Remote version cache** — cache `list_remote()` results to `~/.vex/cache/remote-<tool>.json` with configurable TTL (default 5 min via `cache_ttl_secs` in `config.toml`)
- **`--no-cache` flag** for `vex list-remote` to force fresh fetch
- **Concurrent install lock** — file-based exclusive lock (`~/.vex/locks/`) prevents parallel installs of the same tool@version from corrupting state; fail-fast with clear error message
- **Spinner feedback** during remote API calls (replaces static "Fetching..." text)
- **Download speed display** (`bytes/sec`) in progress bar

### Changed

- Revised public-facing documentation for consistency and clarity: fully translated `CONTRIBUTING.md` to English and standardized wording in `README.md` and `SECURITY.md`.
- Updated public docs to use the real repository URL (`imnotnoahhh/vex`) instead of placeholder links.
- Clarified `vex list-remote` behavior in README (`interactive latest 20` by default, `--all` for full output).
- Added documentation notes for Go/Rust upstream remote-list limits and contributor-facing doc organization rules.
- Added GitHub Releases installation guidance to README, alongside the existing source-build installation path.
- Added a one-line release installer script (`scripts/install-release.sh`) that downloads the matching macOS artifact and updates shell PATH config.
- Clarified uninstall instructions in README with specific shell config lines to remove and binary cleanup steps.

## [0.1.0] - 2026-02-27

### Added

- Multi-language version management: Node.js, Go, Java (Eclipse Temurin), Rust
- Symlink-based version switching (no shim overhead)
- Fuzzy version matching (`node@20` resolves to latest 20.x, `node@lts` to latest LTS)
- Interactive version selection with `vex install <tool>`
- `.tool-versions` support for per-project version pinning
- `vex install` without arguments installs all tools from `.tool-versions`
- `vex local` / `vex global` commands to write `.tool-versions` files
- Shell hooks for zsh and bash (auto-switch on `cd`)
- SHA256 checksum verification for Node.js downloads
- Download progress bar with speed indicator
- `vex current` to show all active versions
- macOS support (Apple Silicon and Intel)
