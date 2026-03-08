# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2026-03-08

### Fixed

- **`vex global` no longer pollutes `~`** — Global version pinning now writes to `~/.vex/tool-versions` instead of `~/.tool-versions`, keeping all vex data under `~/.vex/`
- **`~/.cargo` no longer created in home directory** — Shell hooks now export `CARGO_HOME=$HOME/.vex/cargo` so cargo stores its data inside `~/.vex/cargo/` instead of `~/.cargo/`

### Changed

- **Resolver fallback** — `resolve_versions` and `resolve_version` now fall back to `~/.vex/tool-versions` (global) after traversing the directory tree, replacing the previous `~/.tool-versions` lookup
- **`vex global` help text** — Updated to reflect the new path (`~/.vex/tool-versions`)
- **`vex python` help text** — Expanded with a step-by-step workflow description so users understand the `init → freeze → sync` lifecycle without reading the docs

### Documentation

- Added Python workflow section to `docs/guides/getting-started.md` with a clear 5-step guide (install → init → freeze → commit → sync)
- Added Known Limitations section to `docs/guides/troubleshooting.md`:
  - `~/.cargo` migration instructions (`mv ~/.cargo ~/.vex/cargo`)
  - `~/.cache/node` explanation (npm/pnpm behavior, manual workaround)

## [0.2.0] - 2026-03-08

### Added

- **`vex self-update`** — Update vex itself to the latest GitHub release without reinstalling:
  - Fetches latest release info from GitHub API
  - Compares semver against current binary version
  - Downloads the matching platform asset (`aarch64-apple-darwin` or `x86_64-apple-darwin`)
  - Supports both bare binary and `.tar.gz` release assets
  - Atomically replaces the current executable via `fs::rename`
  - Prints "already up to date" if no newer version exists

- **Python support** — Full Python version management via [python-build-standalone](https://github.com/astral-sh/python-build-standalone):
  - `vex install python@<version>` / `vex use python@<version>` / `vex list python` / `vex list-remote python`
  - Version aliases: `latest`, `stable`, `bugfix`, `security` based on Python's support lifecycle
  - `vex python init` — creates `.venv` in current directory using active Python version
  - `vex python freeze` — locks environment to `requirements.lock` via `pip freeze`
  - `vex python sync` — restores environment from `requirements.lock` (auto-inits `.venv` if missing)
  - Shell hooks extended with `__vex_activate_venv` — auto-activates/deactivates `.venv` on `cd` in zsh, bash, fish, and nushell
  - Checksums verified via `SHA256SUMS` file published alongside each release

### Changed

- **Dynamic binary detection** — Version switching now automatically detects available binaries instead of relying on hardcoded lists:
  - Scans toolchain bin directories for actual executables
  - Only creates symlinks for binaries that exist
  - Automatically cleans up stale symlinks when switching versions
  - Handles version-specific binaries (e.g., corepack in Node.js 24 but not 25+)
- **Node.js 25+ Corepack handling** — Improved support for Node.js 25+ which no longer bundles Corepack:
  - Installation shows helpful message: "Node.js 25+ no longer includes Corepack. To use pnpm or yarn, run: corepack enable pnpm"
  - `vex doctor` no longer reports false warnings for missing corepack in Node.js 25+
  - Corepack automatically available in Node.js 24 and earlier versions
- **Test coverage improvement** — Increased test coverage from 46.99% to 66.51% (+19.52%):
  - Added 25 new unit tests covering core logic (version resolution, file operations, cleanup guards)
  - Added 10 new CLI integration tests for install, global, use --auto, and doctor commands
  - Achieved 100% coverage for 5 core modules (cache, resolver, shell, switcher, tools/mod)
  - Total test count: 181 tests (133 unit + 43 CLI + 5 E2E)
  - Coverage: 828/1245 lines covered
- **Documentation translation** — Translated all Rustdoc comments from Chinese to English across 14 modules for better international accessibility

## [0.1.6] - 2026-03-02

### Added

- **`vex doctor` command** — Health check command that validates:
  - vex installation and PATH configuration
  - Shell hook setup (auto-switch on cd)
  - Installed tool versions and their activation status
  - Binary symlinks integrity
  - Provides actionable suggestions for fixing issues
- **Disk space check** — Installation now checks for at least 500 MB free disk space before downloading, preventing partial installs on full disks
- **Path traversal protection** — Archive extraction now validates all paths to prevent malicious tar files from writing outside the installation directory
- **HTTP timeout configuration** — Network requests now have configurable timeouts:
  - Connection timeout: 30 seconds
  - Total timeout: 5 minutes (suitable for large downloads like JDK)
  - Automatic retry on failure (3 attempts with 2-second intervals)
  - 4xx client errors (e.g., 404) are not retried
- **Fish and Nushell shell support** — Added shell integration for Fish and Nushell:
  - `vex env fish` outputs Fish shell hook
  - `vex env nu` outputs Nushell hook
  - Auto-switch on directory change works in all supported shells
- **Enhanced error messages** — All error types now include actionable troubleshooting suggestions:
  - Network errors suggest checking internet connection and firewall
  - Disk space errors show required vs available space
  - Permission errors provide chmod/chown commands
  - Version not found errors suggest using `vex list-remote`
- **Performance benchmarks** — Added criterion-based benchmarks for:
  - Version file parsing (.tool-versions)
  - Directory traversal for version resolution
  - Symlink creation and updates (version switching)
  - Cache read/write operations
  - Run with `cargo bench` (not executed in CI)
- **Comprehensive Rustdoc documentation** — All 14 modules now have detailed Chinese documentation:
  - Module-level docs explaining purpose and architecture
  - Function docs with parameters, returns, and errors
  - Type docs for structs, enums, and traits
  - Examples and usage notes
- **End-to-end integration tests** — Added 11 comprehensive E2E tests covering:
  - Full workflow: install → activate → uninstall
  - Node.js and Go installation flows
  - Version switching between multiple installed versions
  - Version alias resolution (lts, latest)
  - .tool-versions file parsing and auto-activation
  - local/global command functionality
  - Concurrent installation protection
  - Network-dependent tests marked with `#[ignore]` for CI performance

### Changed

- **64KB buffer size** — Download and checksum calculation now use 64KB buffers for improved performance
- **User-Agent header** — HTTP requests now identify as `vex/<version>` for better upstream analytics
- **Home directory error handling** — Replaced `home_dir().unwrap()` with proper error handling using `VexError::HomeDirectoryNotFound`

### Fixed

- **Network-dependent tests** — Marked Java alias resolution test as `#[ignore]` to prevent CI failures when network is unavailable

## [0.1.5] - 2026-03-01

### Added

- **Colorful terminal output** — Enhanced user experience with color-coded messages:
  - Green for success messages (✓ Installed, ✓ Switched, ✓ Checksum verified)
  - Cyan for action messages (Installing, Downloading, Switching)
  - Yellow for tool names and versions
  - Dimmed for paths and hints
- **Shell cache hint** — `vex use` now displays a note about running `hash -r` if `which` shows old paths, addressing shell command cache issues

### Changed

- Improved visual hierarchy in terminal output with moderate color usage
- Added `owo-colors` dependency for terminal styling (auto-detects TTY)

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
