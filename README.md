<h1 align="center">vex</h1>

<p align="center">
  <strong>A fast, multi-language version manager for macOS</strong>
</p>

<p align="center">
  Symlink-based switching · Node.js / Go / Java / Rust / Python · .tool-versions · Auto-switch on cd
</p>

<p align="center">
  <a href="https://github.com/imnotnoahhh/vex/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/imnotnoahhh/vex/ci.yml?style=flat-square&label=CI" alt="CI">
  </a>
  <a href="https://github.com/imnotnoahhh/vex/releases">
    <img src="https://img.shields.io/github/v/release/imnotnoahhh/vex?style=flat-square" alt="Release">
  </a>
  <a href="https://github.com/imnotnoahhh/vex/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/imnotnoahhh/vex?style=flat-square" alt="License">
  </a>
  <a href="https://github.com/imnotnoahhh/vex/stargazers">
    <img src="https://img.shields.io/github/stars/imnotnoahhh/vex?style=flat-square" alt="Stars">
  </a>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> ·
  <a href="#commands">Commands</a> ·
  <a href="#tool-versions-workflow">.tool-versions Workflow</a> ·
  <a href="#faq">FAQ</a>
</p>

<p align="center">
  <img src="./docs/demo/vex-install.gif" alt="vex install demo" width="980" />
</p>

## Features

- **Symlink-based switching** — version changes take effect instantly, no shim overhead
- **Multi-language** — manage Node.js, Go, Java (Eclipse Temurin), Rust, and Python from one tool
- **Python base + venv integration** — managed per-version base environments for global Python CLIs, plus `vex python init/freeze/sync` for project `.venv` isolation
- **Shell auto-configuration** — `vex init --shell auto` detects and configures your shell automatically (zsh, bash, fish, nushell)
- **Project templates** — `vex init --list-templates` and `vex init --template <name>` bootstrap official starters for Node, Go, Java, Rust, and Python
- **Safe add-only templating** — `vex init --template <name> --add-only` only merges `.tool-versions` and `.gitignore`, then creates missing starter files
- **Fuzzy version matching** — `node@20` resolves to latest 20.x, `node@lts` to latest LTS
- **Version aliases** — `latest`, `lts`, `lts-<codename>`, `stable`, minor version matching
- **Historical Rust stable installs** — `vex list-remote rust` and `vex install rust@1.93.1` resolve against Rust's official archived stable installers for the current macOS architecture, not just the current stable release
- **User-defined aliases** — `vex alias set/list/delete` for custom version shortcuts
- **TUI dashboard** — `vex tui` for interactive version overview and health check
- **Offline mode** — `--offline` flag for cache-only operations, no network required
- **Lockfile support** — `vex lock` generates reproducible `.tool-versions.lock` with checksums
- **Team config sync** — `vex install --from` / `vex sync --from` support local files, `vex-config.toml`, HTTPS team configs, and Git repositories with a safe `[tools]` schema
- **Managed npm globals** — Shell hooks and `vex exec`/`run` export `NPM_CONFIG_PREFIX=$HOME/.vex/npm/prefix` and keep `~/.vex/npm/prefix/bin` on PATH for stable `npm install -g` behavior
- **Auto-export env vars** — Automatic `JAVA_HOME`, `GOROOT`, `CARGO_HOME`, captured user-state env vars, Python base CLI paths, and project `.venv` activation in shell hooks
- **Official Rust extensions** — `vex rust target/component` manages official Rust toolchain extensions such as `rust-src` and iOS std targets
- **Contained user-state capture** — supported language homes, caches, and user bins default into `~/.vex`
- **Explicit home repair** — `vex repair migrate-home` previews and applies safe migrations from legacy home-directory paths
- **One-command upgrade** — `vex upgrade node` installs and switches to the latest version
- **Managed context upgrades** — `vex outdated` inspects the current project/global/active scope, and `vex upgrade --all` upgrades that whole managed set
- **Explicit relink for Node globals** — `vex relink node` rebuilds `~/.vex/bin` after npm adds new executables to the active Node toolchain
- **Transient execution** — `vex exec -- <command>` runs tools in the resolved vex environment without changing global symlinks
- **Project task runner** — `.vex.toml` can define project env vars and named commands for `vex run <task>`
- **Official GitHub Action** — `uses: imnotnoahhh/vex@v1` installs `vex` plus cached toolchains and managed npm globals on macOS GitHub Actions runners
- **`.tool-versions` support** — per-project pinning, auto-switch on `cd`, batch install
- **Project configuration** — `.vex.toml` adds project-local commands, env vars, behavior overrides, and optional network/mirror overrides
- **Smart version filtering** — `vex list-remote node --filter lts` shows only LTS versions
- **Remote version cache** — cached for 5 min by default, configurable via `config.toml`
- **Concurrent install protection** — file-based locking prevents parallel install corruption
- **Checksum verification** — Node.js uses official SHA256 verification; Go/Java/Rust follow upstream checksum metadata availability
- **Parallel downloads** — atomic writes with automatic cleanup, up to 3 concurrent downloads
- **Parallel extraction** — fast archive extraction using parallel file processing
- **Security hardening** — TOCTOU protection, ownership validation, path traversal protection, atomic operations
- **Self-update** — `vex self-update` upgrades vex itself to the latest GitHub release
- **Health check** — `vex doctor` validates installation, PATH, shell hooks, managed npm/Python global bins, and active manager conflicts with actionable fixes
- **Disk space check** — prevents installation when less than 500 MB free space available
- **Machine-readable output** — `--json` for `current`, `list`, `list-remote`, and `doctor`
- **Homebrew support** — optional official tap for brew users, while direct install remains the recommended path
- **Multi-shell support** — zsh, bash, fish, and nushell integration for auto-switching
- **macOS native** — supports both Apple Silicon and Intel macOS environments

## Quick Start

### Install

#### One-line installer (Recommended)

Automatically downloads the correct prebuilt binary for your macOS architecture (`arm64`/`x86_64`), installs to `~/.local/bin/vex`, and updates your shell PATH configuration:

```bash
# Latest release
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash

# Specific tag
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash -s -- --version v1.6.2
```

For auditability, review the script before running:

```bash
curl -fsSL -o install-release.sh https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh
less install-release.sh
bash install-release.sh --help
```

#### Manual download from GitHub Releases

Download the prebuilt binary for your architecture from the [Releases page](https://github.com/imnotnoahhh/vex/releases):

- Apple Silicon (M1/M2/M3): `vex-aarch64-apple-darwin.tar.gz`
- Intel: `vex-x86_64-apple-darwin.tar.gz`

Extract and install:

```bash
tar -xzf vex-*.tar.gz
mkdir -p ~/.local/bin
cp vex-*/vex ~/.local/bin/vex
chmod +x ~/.local/bin/vex

# Add to PATH if not already present
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

#### Homebrew tap (Optional)

If you already use Homebrew, vex can also be installed from the official tap:

```bash
brew install imnotnoahhh/homebrew-vex/vex
```

Direct installation remains the recommended path because it keeps vex completely independent from Homebrew after setup.

#### Build from source

```bash
git clone https://github.com/imnotnoahhh/vex.git
cd vex
cargo build --release && cp target/release/vex ~/.local/bin/vex
```

Verify installation:

```bash
vex --version
```

### Setup

```bash
vex init --shell auto

# Or configure shell hooks manually.
# For zsh:
echo 'eval "$(vex env zsh)"' >> ~/.zshrc
source ~/.zshrc

# For bash:
echo 'eval "$(vex env bash)"' >> ~/.bashrc
source ~/.bashrc

# For fish:
echo 'vex env fish | source' >> ~/.config/fish/config.fish

# For nushell:
vex env nu | save -f ~/.config/nushell/vex.nu
echo 'source ~/.config/nushell/vex.nu' >> ~/.config/nushell/config.nu
```

The generated hook keeps `~/.vex/npm/prefix/bin` and `~/.vex/bin` on `PATH`, runs `vex use --auto` on directory changes, and refreshes the exported activation environment via `vex env <shell> --exports`.

### Usage

```bash
# Install a specific version (fuzzy matching)
# Note: Automatically switches to the installed version
vex install node@20          # → latest 20.x
vex install node@lts         # → latest LTS
vex install node@20.11.0     # → exact version

# Install without switching (preserve current version)
vex install node@20 --no-switch

# Version aliases
vex install node@lts-iron    # → specific LTS codename
vex install go@1.23          # → latest 1.23.x
vex install rust@stable      # → latest stable

# Bootstrap a project starter
vex init --list-templates
vex init --template node-typescript
vex init --template python-venv --add-only

# Sync from a team-managed source
vex sync --from vex-config.toml
vex sync --from https://company.example/vex-config.toml
vex install --from git@github.com:company/vex-config.git

# Upgrade to latest
vex upgrade node
vex upgrade --all

# Show what is behind latest in the current managed context
vex outdated

# List user-defined aliases
vex alias list

# Run a command in the resolved vex-managed environment
vex exec -- node -v

# Run a named task from .vex.toml
vex run test

# Rebuild Node binary links after npm installs a new global CLI
vex relink node

# Preview or apply safe home-directory migrations into ~/.vex
vex repair migrate-home
vex repair migrate-home --apply

# Manage official Rust extensions for the active toolchain
vex rust target list
vex rust target add aarch64-apple-ios aarch64-apple-ios-sim
vex rust component add rust-src

# Switch versions
vex use node@22

# Pin version for current project
vex local node@20.11.0       # writes .tool-versions

# Install everything from .tool-versions
vex install
```

## Commands

For the full CLI reference, including command groups and option details, see [docs/guides/command-reference.md](docs/guides/command-reference.md).

| Command | Description | Example |
|---------|-------------|---------|
| `vex init` | Initialize directory structure | `vex init` |
| `vex init --shell auto` | Initialize and auto-configure shell | `vex init --shell auto` |
| `vex init --shell zsh` | Initialize and configure specific shell | `vex init --shell zsh` |
| `vex init --list-templates` | List built-in project templates | `vex init --list-templates` |
| `vex init --template <name>` | Bootstrap a project starter | `vex init --template rust-cli` |
| `vex init --template <name> --add-only` | Safely merge missing template files into an existing repo | `vex init --template python-venv --add-only` |
| `vex install <tool@version>` | Install a specific version | `vex install node@20` |
| `vex install <tool@version> <tool@version>...` | Install multiple specific versions | `vex install node@20 go@1.24` |
| `vex install` | Install all from `.tool-versions` | `vex install` |
| `vex install --from <source>` | Install from a version file, `vex-config.toml`, HTTPS URL, or Git repo | `vex install --from git@github.com:company/vex-config.git` |
| `vex use <tool@version>` | Switch to installed version | `vex use node@22` |
| `vex use --auto` | Auto-switch from version files | `vex use --auto` |
| `vex relink node` | Rebuild `~/.vex/bin` from the active Node toolchain after new npm global executables appear | `vex relink node` |
| `vex local <tool@version>` | Pin version in `.tool-versions` | `vex local node@20.11.0` |
| `vex global <tool@version>` | Pin version in `~/.vex/tool-versions` | `vex global go@1.23` |
| `vex list <tool>` | List installed versions | `vex list node` |
| `vex list <tool> --json` | List installed versions as JSON | `vex list node --json` |
| `vex list-remote <tool>` | List all remote versions | `vex list-remote node` |
| `vex list-remote <tool> --json` | List remote versions as JSON | `vex list-remote node --json` |
| `vex list-remote <tool> -f lts` | List only LTS versions | `vex list-remote node -f lts` |
| `vex list-remote <tool> -f major` | List latest of each major version | `vex list-remote node -f major` |
| `vex list-remote <tool> --no-cache` | List remote versions (skip cache) | `vex list-remote node --no-cache` |
| `vex upgrade <tool>` | Upgrade to latest version | `vex upgrade node` |
| `vex upgrade --all` | Upgrade every managed tool in the current context | `vex upgrade --all` |
| `vex outdated` | Show managed tools that are behind latest | `vex outdated` |
| `vex outdated --json` | Show outdated status as JSON | `vex outdated --json` |
| `vex prune --dry-run` | Preview cache, stale-lock, and unused-toolchain cleanup | `vex prune --dry-run` |
| `vex gc` | Alias for `vex prune` | `vex gc --dry-run` |
| `vex install --force` | Reinstall a version even if it already exists | `vex install node@20 --force` |
| `vex install --frozen` | Install from version files while strictly enforcing `.tool-versions.lock` | `vex install --frozen` |
| `vex alias set <tool> <alias> <version>` | Set custom version alias | `vex alias set node lts-current 20.11.0` |
| `vex alias list [tool]` | List all aliases | `vex alias list node` |
| `vex alias delete <tool> <alias>` | Delete an alias | `vex alias delete node lts-current` |
| `vex lock` | Generate lockfile from `.tool-versions` | `vex lock` |
| `vex sync --from <source>` | Sync from a version file, `vex-config.toml`, HTTPS URL, or Git repo | `vex sync --from https://company.example/vex-config.toml` |
| `vex sync --frozen` | Install from lockfile | `vex sync --frozen` |
| `vex sync --offline` | Sync using cached metadata and archives only | `vex sync --offline` |
| `vex tui` | Launch interactive dashboard | `vex tui` |
| `vex install --offline` | Install from cache only | `vex install node@20 --offline` |
| `vex exec -- <command>` | Run a command in the resolved vex environment without switching global state | `vex exec -- node -v` |
| `vex run <task> [args...]` | Run a named task from `.vex.toml` | `vex run test -- --nocapture` |
| `vex current` | Show active versions | `vex current` |
| `vex current --json` | Show active versions as JSON | `vex current --json` |
| `vex uninstall <tool@version>` | Uninstall a version | `vex uninstall node@20.11.0` |
| `vex doctor` | Run health check and diagnostics | `vex doctor` |
| `vex doctor --json` | Run health check and emit JSON | `vex doctor --json` |
| `vex doctor --verbose` | Show extra provenance and captured-env details | `vex doctor --verbose` |
| `vex repair migrate-home` | Preview or apply safe legacy home-directory migrations into `~/.vex` | `vex repair migrate-home --apply` |
| `vex self-update` | Update vex itself to the latest release | `vex self-update` |
| `vex env <shell>` | Output shell hook script | `vex env zsh` |
| `vex rust target <subcommand>` | Manage official Rust targets for the active Rust toolchain | `vex rust target add aarch64-apple-ios` |
| `vex rust component <subcommand>` | Manage official Rust components for the active Rust toolchain | `vex rust component add rust-src` |
| `vex python base` | Ensure the active Python base environment exists | `vex python base` |
| `vex python base pip <args>` | Run pip inside the active Python base environment | `vex python base pip install kaggle` |
| `vex python base freeze` | Lock base Python CLI packages to `requirements.lock` inside the base env | `vex python base freeze` |
| `vex python base sync` | Restore base Python CLI packages from the base env lockfile | `vex python base sync` |
| `vex python init` | Create `.venv` in current directory | `vex python init` |
| `vex python freeze` | Lock environment to `requirements.lock` | `vex python freeze` |
| `vex python sync` | Restore environment from `requirements.lock` | `vex python sync` |

## Supported Tools

| Tool | Binaries | Source |
|------|----------|--------|
| Node.js | node, npm, npx (+ corepack in v24 and earlier) | Official binaries |
| Go | go, gofmt | Official binaries |
| Java | java, javac, jar, javadoc + 26 more JDK tools | Eclipse Temurin JDK |
| Rust | rustc, rustdoc, cargo, rustfmt, clippy, rust-analyzer + 5 more | Official stable binaries |
| Python | python3, pip3, python, pip, 2to3, idle3, pydoc3, python3-config | python-build-standalone (astral-sh) |

## Documentation

- User guides and troubleshooting: [docs/README.md](docs/README.md)
- Full command reference: [docs/guides/command-reference.md](docs/guides/command-reference.md)
- Migration and comparison: [docs/guides/migration-comparison.md](docs/guides/migration-comparison.md)
- Benchmark methodology: [docs/guides/benchmark-methodology.md](docs/guides/benchmark-methodology.md)
- Team and CI recommendations: [docs/guides/best-practices.md](docs/guides/best-practices.md)
- Maintainer and contributor docs: [docs/development/README.md](docs/development/README.md)

## Configuration

Global settings live in `~/.vex/config.toml`. Project settings in `.vex.toml` can override behavior and network defaults within that repo. Environment variables still take highest precedence for CI and enterprise shells.

```toml
# ~/.vex/config.toml
cache_ttl_secs = 300

[network]
connect_timeout_secs = 30
read_timeout_secs = 300
download_retries = 3
proxy = "http://proxy.internal:8080"

[mirrors]
node = "https://mirror.example.com/nodejs"
```

```toml
# .vex.toml
[behavior]
auto_switch = true
auto_activate_venv = true

[network]
download_retries = 5
proxy = "http://team-proxy.internal:8080"

[mirrors]
rust = "https://mirror.example.com/rust"

[env]
RUST_LOG = "debug"

[commands]
test = "cargo test"
lint = "cargo clippy --all-targets --all-features -- -D warnings"
```

Use `vex exec` for one-off commands and `vex run` for project tasks:

```bash
vex exec -- python -m pytest
vex run test
```

For more detail, see [docs/guides/configuration.md](docs/guides/configuration.md).

## Project Templates

`vex init` now has two explicit modes:

- `vex init --shell ...` initializes `~/.vex` and shell integration
- `vex init --template ...` bootstraps the current project directory

The built-in core templates are:

- `node-typescript`
- `go-service`
- `java-basic`
- `rust-cli`
- `python-venv`

Template defaults:

- `--dry-run` previews every file without writing anything
- strict mode exits without writing if any target file already exists
- `--add-only` only merges `.tool-versions` and `.gitignore`, then creates any missing starter files

## Team Config Sync

`vex install --from` and `vex sync --from` can now consume:

- a local version file such as `.tool-versions`
- a local `vex-config.toml`
- an HTTPS-hosted `vex-config.toml`
- an HTTPS or SSH Git repository whose root contains `vex-config.toml`

Team config is intentionally narrow and safe:

```toml
version = 1

[tools]
node = "20"
go = "1.24"
python = "3.12"
```

Rules:

- remote team config only supports `[tools]`
- local `.tool-versions` entries override the remote baseline for matching tools
- team config is only loaded when you explicitly pass `--from`
- local `--from` file paths are resolved relative to your current working directory

## GitHub Actions

This repository now publishes a macOS-only composite action:

```yaml
- uses: imnotnoahhh/vex@v1
  with:
    tools: node@20 go@1.24
```

Or:

```yaml
- uses: imnotnoahhh/vex@v1
  with:
    auto-install: true
```

The action:

- installs the latest `vex` release or a requested release tag
- caches `~/.vex/cache` and `~/.vex/toolchains`
- re-runs activation after cache restore so `~/.vex/bin` is ready in `PATH`

## Lockfile Workflow

Lock your toolchain versions for reproducible environments:

```bash
# 1. Pin versions in .tool-versions
vex local node@20.11.0
vex local go@1.23.5

# 2. Generate lockfile with checksums
vex lock
# Creates .tool-versions.lock with SHA256 checksums

# 3. Commit both files
git add .tool-versions .tool-versions.lock
git commit -m "Lock toolchain versions"
```

Teammates can restore the exact environment:

```bash
# Install with frozen lockfile (enforces exact versions)
vex sync --frozen
```

The lockfile includes SHA256 checksums for security and reproducibility.

## User-Defined Aliases

Create custom version shortcuts:

```bash
# Set a global alias (default)
vex alias set node production 20.11.0

# Set a project-local alias
vex alias set --project node lts-current 20.11.0

# List all aliases
vex alias list node

# Use the alias
vex install node@lts-current

# Delete an alias
vex alias delete node lts-current
```

Aliases are stored in:
- Project: `.vex.toml` (committed to git)
- Global: `~/.vex/aliases.toml` (user-specific)

Project aliases override global aliases.

## Fuzzy Version Matching

Version specs don't need to be exact:

```bash
vex install node@20       # latest 20.x.x
vex install node@20.11    # latest 20.11.x
vex install node@20.11.0  # exact version
vex install java@21       # exact (Java uses single numbers)
```

## Version Aliases

| Tool | Aliases |
|------|---------|
| Node.js | `latest`, `lts`, `lts-<codename>` (e.g. `lts-iron`) |
| Go | `latest`, `<major>.<minor>` (e.g. `1.23` → latest 1.23.x) |
| Java | `latest`, `lts` |
| Rust | `latest`, `stable` |
| Python | `latest`, `stable`, `bugfix`, `security` |

```bash
# Inspect remote versions before choosing a built-in alias
vex list-remote node --filter lts
```

## `.tool-versions` Workflow

```bash
# Pin versions for your project
vex local node@20.11.0
vex local go@1.23.5

# .tool-versions is created automatically:
# go 1.23.5
# node 20.11.0

# Teammates clone and run:
vex install          # installs everything from .tool-versions

# With shell hook enabled, versions auto-switch on cd
cd my-project        # → switches to node 20.11.0, go 1.23.5
```

## Python Workflow

Python binaries come from [python-build-standalone](https://github.com/astral-sh/python-build-standalone) standard `install_only` CPython packages — prebuilt, standalone binaries with no compilation needed. `vex` does not currently manage free-threaded Python variants.

```bash
# 1. Install a Python version
vex install python@3.12   # or: python@latest, python@bugfix, python@security

# 2. Activate it
vex use python@3.12

# 3. Optional: install global Python CLIs into the managed base env
#    These are available when no project .venv is active.
vex python base
vex python base pip install kaggle
kaggle --version

# 4. Create a project venv
#    Uses ~/.vex/bin/python3 (the active vex-managed python), falls back to system python3
cd my-project
vex python init      # runs: python3 -m venv .venv
                     # also writes python version to .tool-versions

# 5. Install packages and lock them
pip install requests flask
vex python freeze    # runs: pip freeze > requirements.lock

# 6. Commit both files
git add .tool-versions requirements.lock
```

On a new machine or for a teammate:

```bash
vex install python@3.12
cd my-project
vex python sync      # auto-creates .venv if missing + pip install -r requirements.lock
```

The shell hook automatically refreshes `PATH`, `VIRTUAL_ENV`, and captured tool env vars when you `cd` into or out of a project — no manual `source .venv/bin/activate` needed.

Python has two managed dependency scopes:

- `~/.vex/python/base/<version>` is the per-version base environment. Use it for user-level Python CLIs such as `kaggle`, `black`, or `pipx` alternatives when no project `.venv` is active.
- `project/.venv` is the project environment. When the shell hook activates `.venv`, the Python base `bin` directory is intentionally hidden so base packages and CLI scripts do not leak into the project.

If you install a CLI with `vex python base pip install kaggle`, it is available from the shell outside project virtual environments. Inside a project, install project-specific dependencies into `.venv` and lock them with `vex python freeze`.

`requirements.lock` is generated by `pip freeze` and pins all packages including transitive dependencies. Commit it to git for reproducible environments.

## How It Works

vex uses symlinks + PATH prepending. No shims, no runtime overhead.

```
~/.vex/bin/node  →  ~/.vex/toolchains/node/20.11.0/bin/node
                              ↓ (vex use node@22)
~/.vex/bin/node  →  ~/.vex/toolchains/node/22.0.0/bin/node
```

Switching versions just updates symlinks — instant and shell-restart-free.

## Comparison

| | vex | nvm | fnm | asdf | mise |
|---|---|---|---|---|---|
| Multi-language | ✅ | ❌ Node only | ❌ Node only | ✅ | ✅ |
| Python venv management | ✅ built-in | ❌ | ❌ | ❌ | ❌ |
| No shims | ✅ symlinks | ✅ | ✅ | ❌ shims | ❌ shims |
| .tool-versions | ✅ | ❌ | ❌ | ✅ | ✅ |
| Auto-switch on cd | ✅ | ❌ | ✅ | ✅ | ✅ |
| Zero home dir pollution | ✅ all in ~/.vex | ❌ | ❌ | ❌ | ❌ |
| Self-update | ✅ built-in | ❌ | ❌ | ❌ | ❌ |
| Implementation | Rust | Shell | Rust | Shell | Rust |

**Why no shims matters**: asdf and mise insert a shim binary in front of every command. Every time you run `node`, the shim wakes up, looks up the version, then execs the real binary. vex skips this entirely — `~/.vex/bin/node` is a direct symlink to the real binary. Zero overhead, no startup tax.

**Why zero home dir pollution matters**: most version managers scatter files across `~/.nvm`, `~/.cargo`, `~/.cache/node`, `~/.tool-versions`, etc. vex keeps everything under `~/.vex/` — one directory, easy to back up, easy to nuke.

## Directory Layout

```
~/.vex/
├── bin/            # Symlinks (added to PATH)
├── toolchains/     # Installed versions
│   ├── node/
│   │   ├── 20.11.0/
│   │   └── 22.0.0/
│   ├── go/
│   ├── java/
│   └── rust/
├── current/        # Active version symlinks
├── cache/          # Download cache + remote version cache
├── locks/          # Install lock files (concurrent protection)
└── config.toml     # Configuration (e.g. cache_ttl_secs)
```

## FAQ
Run `vex doctor` to perform a comprehensive health check. It validates:
- vex installation and PATH configuration
- Shell hook setup (auto-switch on cd)
- Installed tool versions and activation status
- Binary symlinks integrity
- Managed Python base environment health and project `.venv` isolation
- Provides actionable suggestions for fixing issues

**Why does `vex list-remote go` not show every historical Go release?**
Go remote listings are constrained by upstream API policy and usually focus on active maintenance lines.

**Why does `vex list-remote rust` show limited choices?**
Current Rust support reads `channel-rust-stable.toml` and only targets the stable channel's current release.

**How do I upgrade vex itself?**
Run `vex self-update`. It fetches the latest release from GitHub, downloads the binary for your architecture, and replaces the current executable atomically.

**How do I upgrade to the latest version of a tool?**
Use `vex upgrade <tool>`. It installs the latest version and switches to it automatically.

**Can it coexist with nvm/fnm?**
Technically yes, but not recommended — PATH conflicts are likely.

**Windows/Linux support?**
macOS only for now.

**How to uninstall vex?**

```bash
# 1. Remove these lines from ~/.zshrc (or ~/.bashrc):
#    eval "$(vex env zsh)"
#    export PATH="$HOME/.vex/bin:$PATH"

# 2. Remove vex data and binary
rm -rf ~/.vex
rm -f ~/.local/bin/vex
```

## Development

```bash
git clone https://github.com/imnotnoahhh/vex.git
cd vex
cargo build
cargo test

# Optional: run network-dependent tests explicitly
cargo test --features network-tests

# CI-aligned smoke tests
VEX_BIN="$(pwd)/target/debug/vex" bash scripts/test-management-features.sh
VEX_BIN="$(pwd)/target/debug/vex" bash scripts/test-shell-hooks.sh
VEX_BIN="$(pwd)/target/debug/vex" bash scripts/test-rust-extensions-live.sh
```

### Documentation

Generate comprehensive API documentation with custom styling:

```bash
# Quick command
make docs

# Or manually
RUSTDOCFLAGS="--html-in-header docs/header.html" cargo doc --no-deps
cp docs/custom.css target/doc/
open target/doc/vex/index.html
```

The documentation includes:
- Pure English documentation for all modules
- Custom theme with improved readability
- Enhanced code highlighting and styling
- Comprehensive module and function documentation

### Running Benchmarks

Performance benchmarks are available to measure key operations:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench bench_parse_tool_versions

# Generate detailed reports (saved to target/criterion/)
cargo bench -- --verbose
```

Benchmarked operations:
- Version file parsing (`.tool-versions`)
- Directory traversal for version resolution
- Symlink creation and updates (version switching)
- Cache read/write operations
- Parallel vs sequential file extraction

Note: Benchmarks are not run in CI to keep build times fast. Run them locally to measure performance improvements.

For reporting guidance and fair comparison rules, see [docs/guides/benchmark-methodology.md](docs/guides/benchmark-methodology.md).

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Roadmap

### Planned Features

- **Linux support** — Extend to Linux distributions (Ubuntu, Debian, Fedora, Arch)
- **Windows support** — Windows compatibility with junction points
- **Plugin system** — Allow community-contributed tool adapters
- **Version constraints** — Support version ranges in `.tool-versions` (e.g., `node >=20.0.0 <21.0.0`)
- **Global default versions** — Set default versions without `.tool-versions` file
- **Parallel installations** — Install multiple tools concurrently
- **Update notifications** — Notify when new tool versions are available
- **Self-update** — ✅ Done (`vex self-update`)

### Future Considerations

- **Additional languages**: Ruby, PHP, Elixir, Zig
- **Custom binary sources**: Support for private registries
- **Version pinning strategies**: Lock files for reproducible builds
- **Integration with CI/CD**: GitHub Actions, GitLab CI support

See [GitHub Issues](https://github.com/imnotnoahhh/vex/issues) for detailed feature requests and discussions.

## Contributors

Thanks to everyone who has contributed to vex!

<!-- ALL-CONTRIBUTORS-LIST:START -->
- [Noah Qin](https://github.com/imnotnoahhh) - Creator and maintainer
<!-- ALL-CONTRIBUTORS-LIST:END -->

Want to contribute? Check out [CONTRIBUTING.md](CONTRIBUTING.md) to get started!

## License

[MIT](LICENSE) © 2026 Noah Qin

## Acknowledgements

Inspired by [nvm](https://github.com/nvm-sh/nvm), [fnm](https://github.com/Schniz/fnm), [asdf](https://github.com/asdf-vm/asdf), and [mise](https://github.com/jdx/mise).
