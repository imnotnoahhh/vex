# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Machine-readable command output** — Added `--json` support for `vex current`, `vex list`, `vex list-remote`, and `vex doctor`, backed by shared output/report models for CLI, CI, and editor integrations.
- **Managed upgrade workflows** — Added `vex outdated` to inspect the current managed scope and `vex upgrade --all` to upgrade the full active/project/global context in one command.
- **Workspace cleanup commands** — Added `vex prune` plus the `vex gc` alias to preview or remove cached downloads, stale locks, and unreferenced toolchains while preserving active and pinned versions.
- **Project runtime commands** — Added `vex exec -- <command>` for transient execution without switching global symlinks and `vex run <task>` for named commands declared in `.vex.toml`.
- **Project configuration support** — Added `.vex.toml` parsing for project-local behavior, network overrides, mirrors, environment variables, and named tasks.
- **Release postflight automation** — Added `.github/workflows/release-postflight.yml` to validate release notes, smoke-test published macOS binaries, and update the official Homebrew tap when credentials are available.
- **Homebrew tap packaging support** — Added formula rendering automation plus packaging docs for the optional `imnotnoahhh/homebrew-vex` tap.
- **Management feature smoke coverage** — Added `scripts/test-management-features.sh` plus a dedicated CI job to exercise the real behavior of `--json`, `outdated`, `upgrade --all`, `prune`/`gc`, `.vex.toml`, `vex exec`, and `vex run` in isolated homes instead of relying only on unit tests or format/lint checks.

### Fixed

- **Configuration schema errors now surface consistently** — Commands that load effective settings, including `vex current --json`, now fail fast when `~/.vex/config.toml` or a project `.vex.toml` contains invalid types instead of silently falling back to defaults. `vex doctor --json` now validates the typed config schema as well, so malformed settings are reported as configuration warnings instead of being marked valid.
- **Python release metadata fetch reliability** — Reworked `src/tools/python.rs` to resolve python-build-standalone versions via the latest release tag plus `SHA256SUMS` instead of decoding GitHub's large `releases/latest` JSON payload. This fixes transient `error decoding response body` failures seen in strict macOS CI and other network-sensitive environments when installing Python.
- **Python lifecycle alias correctness** — Fixed Python alias resolution so `python@latest`, `python@stable`, and `python@bugfix` now resolve to the current official bugfix branch, while `python@security` follows the official security-only branch. This corrects stale lifecycle mappings that previously resolved `latest` to `3.13.12` instead of `3.14.3`.
- **Strict macOS Java probe false negative** — Updated `scripts/test_vex_release_strict.py` so `serialver` is validated against its real help output format on modern JDKs, eliminating a strict CI false failure on Java 25.
- **Management smoke tests no longer pin a stale Python patch release** — `scripts/test-management-features.sh` now derives both the newest and older stable Python versions from `vex list-remote python --json` instead of hardcoding `3.13.12`, so the feature smoke job stays aligned with upstream python-build-standalone releases.

### Changed

- **Configuration system overhaul** — Reworked configuration handling into a typed settings model with defaults, `~/.vex/config.toml`, project `.vex.toml`, and environment-variable overrides. Added support for mirrors, proxy settings, request timeouts, retry controls, redirect limits, default shell selection, and non-interactive mode.
- **Enterprise/network behavior is now consistent** — Node, Go, Java, Rust, Python, the downloader, and release metadata fetches now share configurable HTTP clients so proxy, timeout, retry, and project-level network settings apply consistently across metadata resolution and archive downloads.
- **Doctor coverage expanded** — `vex doctor` now validates PATH priority, duplicate shell hooks, cache integrity, global `tool-versions`, project `.vex.toml`, and the effective configuration surface in addition to existing filesystem and binary checks.
- **Python lifecycle status is now dynamic** — Python support phases are now derived from the official Python version-status page at runtime, with a built-in fallback when the upstream page is unavailable. This keeps `bugfix`, `security`, and future branch transitions aligned with the official Python release lifecycle.
- **Python remote version labels now use `Status:` instead of `LTS:`** — `vex list-remote python` now displays lifecycle phases using Python's official terminology (`feature`, `bugfix`, `security`, `end-of-life`) instead of reusing the generic `LTS` label used by Node.js and Java.

## [1.1.1] - 2026-03-12

### Fixed

- **[P0] Rust historical version checksum verification** — Fixed `src/tools/rust.rs` so `vex install rust@<version>` fetches the checksum for the exact requested version instead of incorrectly reusing the current stable manifest. This fixes checksum mismatches for installs such as `rust@1.93.1`.

### Changed

- **Documentation refresh for v1.1.1** — Updated installation and release examples to `v1.1.1`, corrected cache-path and cache-TTL guidance, and refreshed validation coverage docs to match the current macOS strict test suite.
- **Repository layout cleanup** — Moved maintainer-focused docs into `docs/development/` and moved the interactive test helper into `scripts/` to keep the repository root focused on user-facing files and core project metadata.

## [1.1.0] - 2026-03-11

### Added

- **[P1] Automatic broken installation repair** — Added post-update detection and repair in `src/updater.rs` that automatically identifies broken installations from old vex versions (< 1.1.0). After `vex selfupdate`, scans all installed toolchains for 0-byte files (symptom of the old symlink bug), prompts user to reinstall affected versions, and runs `vex install` with `--no-switch` to repair them. Includes version comparison logic to only trigger for upgrades from pre-1.1.0 versions.
- **[P3] Security test suite** — Created `scripts/test-security.sh` to validate security controls: path traversal protection, symlink target validation, directory ownership verification, checksum validation, HTTP redirect limits, disk space checks, lock file PID validation, version input sanitization, atomic operations, and secure temporary file handling. Comprehensive coverage of all security features with automatic cleanup and detailed reporting.
- **[P3] Performance test suite** — Created `scripts/test-performance.sh` to benchmark critical operations: version switching speed (target: <1s single, <500ms average), cache acceleration (target: 5x+ speedup), binary execution latency (target: <10ms), concurrent operations (target: <3s), and memory usage (target: <100MB). Includes automatic cleanup, detailed metrics, and pass/fail statistics.
- **[P3] Automatic version rollback** — Added rollback mechanism in `src/switcher.rs` that automatically restores the previous version if switching fails. Saves current version before attempting switch, extracts switch logic to `perform_switch()` function, and rolls back on failure with detailed logging and user notifications. Eliminates need for manual recovery after failed switches.
- **[P3] Centralized configuration management** — Created `src/config.rs` module to consolidate all configuration constants (HTTP timeouts, buffer sizes, retry settings, concurrent download limits, redirect limits, disk space requirements, cache TTL, directory names). Provides helper functions for directory paths and supports `VEX_HOME` environment variable for custom installation paths. Eliminates scattered configuration across modules.
- **[P3] Structured logging framework** — Integrated `tracing` and `tracing-subscriber` for structured logging throughout the codebase. Added `src/logging.rs` module with environment variable configuration via `VEX_LOG` (trace/debug/info/warn/error). Instrumented critical operations in downloader, installer, and switcher modules. Usage: `VEX_LOG=debug vex install node@20`. Documentation in `docs/logging.md`.
- **[P1] Shell integration test suite** — Created `scripts/test-shell-hooks.sh` to validate shell hook generation for all supported shells (zsh, bash, fish, nushell). Tests verify presence of auto-switch functions (`__vex_use_if_found`) and Python venv activation hooks (`__vex_activate_venv`).
- **[P2] Cache TTL validation** — Added range validation in `src/cache.rs` for cache TTL configuration. Values must be between 60 seconds (1 minute) and 3600 seconds (1 hour). Invalid values fall back to default 300 seconds with a warning message.
- **[P1] Doctor binary runnability check** — Enhanced `vex doctor` command in `src/main.rs:1473` with step 8 that tests if binaries can actually execute. Tests each binary with `--version`, `--help`, or tool-specific flags (e.g., Go's `version`, Java's `-version`) with 2-second timeout. Detects corrupted binaries that pass file checks but fail to run.
- **[P2] Error handling test suite** — Added error handling tests to `scripts/test-features.sh` covering network errors (invalid versions), unsupported tools, and malicious input (path traversal attempts).
- **[P2] Version file workflow test suite** — Added `.tool-versions` workflow tests to `scripts/test-features.sh` covering `vex local` file creation, content validation, and batch installation from version files.
- **[P2] CI bash test integration** — Added two new CI jobs in `.github/workflows/ci.yml`: `bash-tests` runs `scripts/test-features.sh` on macOS, and `shellcheck` validates all shell scripts on Ubuntu. Both run automatically on push and PR.

### Changed

- **[P3] Test coverage expansion** — Increased test suite from 293 to 299 tests with 7 new unit tests covering config.rs (HTTP/disk/concurrency settings, VEX_HOME environment variable, boundary conditions) and logging.rs (environment filter creation, custom log levels). Achieves comprehensive coverage of configuration management and logging framework modules with focus on error recovery scenarios and edge cases.
- **[P1] Disk space check accuracy** — Improved disk space validation in `src/installer.rs` with precise extraction size estimation. Added `estimate_extraction_size()` function that reads tar headers to calculate actual decompressed size. Check now runs after download (post-checksum) instead of before, requiring `estimated_size + 500MB` instead of fixed 1.5GB. Error messages now show MB units for better precision.
- **[P1] Python version support status** — Updated Python version lifecycle mapping in `src/tools/python.rs:48-53` to reflect current support phases as of 2026-03-10:
  - Python 3.12 moved from bugfix to security-only phase
  - Python 3.14 moved from prerelease to bugfix phase
  - `--filter bugfix` now shows 3.14 and 3.13
  - `--filter security` now shows 3.12, 3.11, and 3.10
- **[P2] Download retry exponential backoff** — Improved retry strategy in `src/downloader.rs` to use exponential backoff (1s, 2s, 4s) instead of fixed intervals, reducing server load during transient failures.

### Fixed

- **[P0] Archive symlink extraction** — Fixed critical bug in `src/installer.rs:187-247` where symlink entries in tar archives were treated as regular files, causing Node.js npm/npx/corepack to be written as 0-byte empty files. Now correctly detects symlink entry types and creates proper symbolic links using `std::os::unix::fs::symlink()`. Rejects absolute path symlink targets for security. Fixes all tools using symlinks (Node.js, Python, Rust).
- **[P2] E2E test CLI parameter** — Updated `tests/e2e_test.rs:334` to use `--filter all` instead of deprecated `--all` flag in `test_e2e_list_remote_command`, aligning with current CLI interface.
- **[P1] Stale lock file cleanup** — Enhanced `src/lock.rs` with PID-based lock validation. Lock files now contain the holding process's PID. When acquiring a lock, vex checks if the PID is still running using `libc::kill(pid, 0)` and automatically cleans up stale locks from crashed processes, eliminating manual cleanup.
- **[P1] Feature test script robustness** — Fixed 7 issues in `scripts/test-features.sh`: added error handling (`set -euo pipefail`), replaced unsafe `eval` with `bash -c`, added cleanup trap mechanism, and corrected test logic for npm/npx symlinks and version filtering.
- **[P0] Temporary file cleanup reliability** — Replaced manual cleanup in `src/downloader.rs` with `tempfile` crate's RAII pattern using `NamedTempFile`. Temporary files are now automatically cleaned up on drop, even during interruptions (Ctrl+C) or panics, preventing disk space leaks.
- **[P0] Exact version offline support** — Fixed `resolve_fuzzy_version()` in `src/tools/mod.rs:100-114` to check for exact versions (e.g., `20.11.0`) before making network requests. Commands like `vex use node@20.11.0` now work offline when the version is already installed, eliminating unnecessary network dependency.
- **[P0] Python 2to3 symlink placeholder** — Fixed incorrect format string in `src/tools/python.rs:276` that generated `2to33.12` instead of `2to3-3.12`, causing the 2to3 command to remain as an empty placeholder. Now correctly creates symlink to version-specific binary (e.g., `2to3` → `2to3-3.12`).
- **[P1] Parallel extraction error reporting** — Improved error handling in `src/installer.rs:284-293` to report all extraction errors instead of only the first one. Error messages now show the total count and full list of failures for better debugging.

### Security

- **[P1] Symlink target path traversal protection** — Enhanced archive extraction in `src/installer.rs:218-241` to validate symlink targets. Rejects symlinks containing `..` (ParentDir) components and absolute paths, preventing zip-slip variant attacks where malicious archives create symlinks pointing outside the installation directory (e.g., `../../../etc/passwd`).
- **[P2] Install script checksum verification** — Added SHA256 checksum validation to `scripts/install-release.sh`. Downloads `.sha256` file from GitHub releases and verifies binary integrity using `shasum -a 256`. Installation fails if checksum mismatch is detected. Falls back with warning if checksum file is unavailable.
- **[P2] Version string input validation** — Added `validate_version_format()` in `src/resolver.rs` to sanitize version inputs from `.tool-versions` files. Rejects path traversal attempts (`../`, `/`, `\`), command injection characters (`;`, `|`, `&`), and excessively long strings (>64 chars). Invalid entries are skipped with warnings.
- **[P0] TOCTOU race condition in symlink switching** — Fixed time-of-check-time-of-use vulnerability in `src/switcher.rs:61-80` by replacing `fs::metadata()` with `File::open().metadata()`. Now uses fstat on file descriptor instead of stat on path, preventing attackers from replacing directories between ownership check and symlink creation.
- **[P2] HTTP redirect limit** — Added redirect policy in `src/downloader.rs` to limit maximum redirects to 10, preventing malicious redirect attacks and infinite redirect loops.

## [1.0.0] - 2026-03-10

### Breaking Changes

- **Auto-switch after install** — `vex install` now automatically switches to the installed version:
  - After installation completes, the new version is immediately activated
  - Use `--no-switch` flag to preserve the old behavior (install without switching)
  - This improves user experience by eliminating the need for a separate `vex use` command
  - **Migration**: If you rely on installing without switching, add `--no-switch` to your scripts
  - Example: `vex install node@20 --no-switch` to install without activating

### Security

- **[P0] TOCTOU race condition in symlink switching** — Fixed time-of-check-time-of-use vulnerability in `switcher.rs`:
  - Use UUID v4 for random temporary filenames instead of predictable `.tmp` extension
  - Add ownership verification to prevent privilege escalation attacks
  - Verify toolchain directory owner matches current user before creating symlinks
- **[P0] Atomic write protection** — All downloads now use UUID-based temporary files to prevent corruption
- **[P0] Directory ownership validation** — Added Unix UID checks to prevent malicious symlink attacks
- **[P0] Secure temporary file handling** — Automatic cleanup of temporary files on failure

### Added

- **Shell auto-configuration** — `vex init --shell auto` automatically detects and configures your shell:
  - Auto-detect shell from `$SHELL` environment variable or config files
  - Support for zsh, bash, fish, and nushell
  - `--dry-run` flag to preview changes without modifying files
  - Checks if vex is already configured to avoid duplicate entries
  - Interactive prompts for shell configuration during installation
- **list-remote filtering** — `vex list-remote <tool> --filter <type>` for targeted version queries:
  - `--filter lts` - Show only LTS versions
  - `--filter major` - Show latest version of each major release
  - `--filter latest` - Show only the latest version
  - `--filter all` - Show all versions (default)
- **Parallel download support** — Downloads now use atomic writes with UUID-based temporary files:
  - Write to `.tmp.{uuid}` first, then atomically rename to final destination
  - Automatic cleanup of temporary files on failure
  - `download_parallel()` function supports up to 3 concurrent downloads
  - Prevents file corruption from interrupted downloads
- **Parallel extraction** — Archive extraction now processes files in parallel:
  - Directories created sequentially to avoid race conditions
  - Files extracted in parallel using rayon for improved performance
  - All path safety validations preserved (path traversal protection)
  - Maintains file permissions during parallel extraction
- **Homebrew integration** — Official Homebrew tap for easy installation:
  - `homebrew/vex.rb.template` - Formula template for releases
  - `homebrew/README.md` - Installation and maintenance guide
  - Automated formula generation for new releases
- **macOS CI matrix** — GitHub Actions now test on multiple macOS versions:
  - macOS 14 (Apple Silicon M-series)
  - macOS 13 (Intel x86_64)
  - Parallel testing across Ubuntu and macOS
  - Platform-specific caching for faster builds
- **Performance benchmarks** — Added benchmarks for parallel vs sequential extraction:
  - `bench_parallel_extraction` measures parallel file writing performance
  - `bench_sequential_extraction` provides baseline comparison
  - Run with `cargo bench` to measure improvements

### Changed

- **Python binary coverage** — Expanded from 2 to 8 binaries for complete toolchain support:
  - Now includes: python3, pip3, python, pip, 2to3, idle3, pydoc3, python3-config
  - Automatically links version-specific binaries (e.g., python3.12, pip3.12)
  - Ensures all Python development tools are accessible after installation
- **Download module** — Enhanced with atomic write support and parallel capabilities:
  - All downloads now use temporary files with UUID naming to avoid conflicts
  - Failed downloads automatically clean up temporary files
  - Maximum concurrent downloads limited to 3 for optimal performance
- **Installer module** — Improved extraction performance:
  - Archive entries read into memory first, then processed in parallel
  - File permissions preserved during parallel extraction
  - Error collection from parallel operations for better debugging
- **Shell module** — New helper functions for shell detection and configuration:
  - `detect_shell()` - Auto-detect current shell
  - `get_shell_config_path()` - Get config file path for shell
  - `is_vex_configured()` - Check if vex hook is already present
- **Init command** — Enhanced with shell configuration options:
  - `--shell auto` - Auto-detect and configure shell
  - `--shell skip` - Skip shell configuration (default)
  - `--dry-run` - Preview changes without modifying files

### Performance

- Archive extraction speed improved significantly for large toolchains (Node.js, Java JDK)
- Download reliability improved with atomic writes preventing partial file corruption
- Parallel file operations reduce installation time for multi-file archives
- CI build times reduced with platform-specific caching

## [0.2.3] - 2026-03-08

### Fixed

- **`vex self-update` downloading .sha256 checksum files** — Fixed asset selection logic that incorrectly matched `.tar.gz.sha256` and `.tar.xz.sha256` files instead of actual binaries. Now explicitly excludes files ending with `.sha256`.
- **Homebrew liblzma dependency** — Statically link `liblzma` to eliminate runtime dependency on Homebrew's `xz` library. Binaries now work on systems without Homebrew installed.

### Changed

- `xz2` crate now uses `static` feature to bundle `liblzma` into the binary

### Migration Note

**Users on v0.2.2 or earlier must manually install v0.2.3** due to bugs in previous `self-update` implementations. After upgrading to v0.2.3, `vex self-update` will work correctly for all future updates.

## [0.2.2] - 2026-03-08

> **⚠️ CRITICAL BUG — DO NOT USE** — This release has two critical issues:
> 1. `vex self-update` downloads `.sha256` checksum files instead of binaries
> 2. Binary has Homebrew `liblzma` dependency, fails on systems without Homebrew
>
> Use v0.2.3 instead.

### Fixed

- **`vex self-update` exec format error** — Self-update now correctly handles `.tar.xz` release assets. Previously, `.tar.xz` was not excluded from the "bare binary" filter and was written directly to disk without extraction, causing `exec format error` on next run. The updater now prefers `.tar.xz` → `.tar.gz` → bare binary, and extracts accordingly.
- **Auto-migrate `~/.tool-versions` → `~/.vex/tool-versions`** — On first run after upgrading from v0.2.0 or earlier, vex automatically detects `~/.tool-versions` and moves it to `~/.vex/tool-versions`. A one-line notice is printed. No manual steps required.

### Added

- `xz2` dependency for `.tar.xz` archive extraction in `vex self-update`

## [0.2.1] - 2026-03-08

> **⚠️ CRITICAL BUG — DO NOT USE** — This release has a broken `vex self-update` that writes corrupt binaries. Use v0.2.2 instead.

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
