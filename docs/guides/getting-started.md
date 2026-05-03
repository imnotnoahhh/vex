# Getting Started with vex

Welcome to vex! This guide will help you get up and running quickly.

## What is vex?

vex is a fast, multi-language version manager for macOS that lets you:

- Install and manage multiple versions of Node.js, Go, Java, Rust, and Python
- Switch between versions instantly (no shim overhead)
- Pin versions per project with `.tool-versions` files
- Auto-switch versions when you `cd` into a project
- Manage Python virtual environments with built-in venv integration

## Quick Start

### 1. Install vex

The easiest way to install vex is with the one-line installer:

```bash
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash
```

This will:
- Download the correct binary for your Mac (Apple Silicon or Intel)
- Install to `~/.local/bin/vex`
- Add vex to your PATH

### 2. Initialize vex

```bash
vex init --shell auto
```

This creates the `~/.vex` directory structure where vex stores installed versions and sets up shell integration in one step.

### 3. Set up shell integration

For automatic version switching when you `cd` into a project:

**For zsh** (default on macOS):
```bash
echo 'eval "$(vex env zsh)"' >> ~/.zshrc
source ~/.zshrc
```

**For bash**:
```bash
echo 'eval "$(vex env bash)"' >> ~/.bashrc
source ~/.bashrc
```

**For fish**:
```bash
echo 'vex env fish | source' >> ~/.config/fish/config.fish
```

**For nushell**:
```bash
vex env nu | save -f ~/.config/nushell/vex.nu
echo 'source ~/.config/nushell/vex.nu' >> ~/.config/nushell/config.nu
```

### 4. Install your first tool

Let's install Node.js:

```bash
vex install node@20
```

vex will:
1. Download the official Node.js binary
2. Verify the checksum
3. Extract and install it
4. Automatically switch to the new version

### 5. Verify installation

```bash
node --version
# v20.11.0

npm --version
# 10.2.4
```

### 6. Bootstrap a project template

Use the built-in starters to create a repo with `.tool-versions`, `.vex.toml`, and minimal source files:

```bash
vex init --list-templates
vex init --template node-typescript
```

For an existing repository, use safe add-only mode:

```bash
vex init --template python-venv --add-only
```

`--add-only` only merges `.tool-versions` and `.gitignore`, then creates missing starter files. If a non-mergeable file already exists, vex exits without partial writes.

## Basic Commands

### Install a tool

```bash
# Specific version
vex install node@20.11.0

# Fuzzy version (latest 20.x)
vex install node@20

# Version alias
vex install node@lts
```

### Switch versions

```bash
vex use node@22
```

### List installed versions

```bash
vex list node
```

### List available versions

```bash
# Show all available versions
vex list-remote node

# Show only the newest patch per major line
vex list-remote node --filter major
```

### Show current versions

```bash
vex current
vex current --json
```

### Uninstall a version

```bash
vex uninstall node@20.11.0
```

### Health check

```bash
vex doctor
vex doctor --json
```

This validates your installation and provides fixes for any issues.

### Script-friendly output

For CI, IDEs, or shell scripts, use JSON output:

```bash
vex list node --json
vex list-remote node --json
vex current --json
vex doctor --json
```

### Upgrade and drift checks

Use `vex outdated` to see whether the current managed context is behind latest, then upgrade one tool or the entire managed set:

```bash
vex outdated
vex outdated --json
vex upgrade node
vex upgrade --all
vex prune --dry-run
vex gc --dry-run
```

### Transient execution and project tasks

Use `vex exec` when you want the right toolchain environment for one command without switching global symlinks:

```bash
vex exec -- node -v
vex exec -- python -m pytest
```

When Node is active, `vex` also prefers the nearest `node_modules/.bin` before shared npm globals. That keeps commands such as `vite`, `eslint`, and `tsc` pointed at the project-installed version when you run them directly, through `vex exec`, or through `vex run`.

Use `vex globals` when a command resolves differently than expected:

```bash
vex globals --verbose
vex globals npm --json
vex globals pip
vex globals go --json
```

It lists shared npm globals, Python base/user environments, Go, and Cargo managed global CLIs, plus Maven/Gradle CLI and cache state with active version-source hints.

Use `.vex.toml` plus `vex run` for repeatable project tasks:

```toml
[commands]
test = "cargo test"
lint = "cargo clippy --all-targets --all-features -- -D warnings"
```

```bash
vex run test
vex run lint
```

## Python Workflow

Python support uses [python-build-standalone](https://github.com/astral-sh/python-build-standalone) standard `install_only` CPython packages — prebuilt binaries with no compilation needed. `vex` currently targets the standard package line, not free-threaded variants.

### Step 1 — Install Python globally

```bash
vex install python@3.12   # or: python@latest, python@stable
vex global python@3.12    # set as global default
```

### Step 2 — Install optional global Python CLIs

Global Python CLIs live in the active version's base environment, similar to a small `conda base` for user tools:

```bash
vex use python@3.12
vex python base pip install kaggle
kaggle --version
```

When no project `.venv` is active, the shell hook exposes `~/.vex/python/base/<version>/bin`. When a project `.venv` is active, that base `bin` path is hidden so base packages do not leak into the project.

### Step 3 — Set up a project

```bash
cd my-project
vex python init
```

This creates `.venv` in the current directory using the active vex-managed Python, and records the version in `.tool-versions`.

### Step 4 — Install packages and lock them

```bash
source .venv/bin/activate   # or let the shell hook do it automatically on next cd
pip install requests flask
vex python freeze            # writes requirements.lock
```

### Step 5 — Commit

```bash
git add .tool-versions requirements.lock
git commit -m "pin python and dependencies"
```

### Step 6 — Restore on another machine

```bash
vex install python@3.12
cd my-project
vex python sync   # creates .venv if missing, then pip install -r requirements.lock
```

### Auto-activation

With the shell hook enabled (`eval "$(vex env zsh)"`), the `.venv` is automatically activated when you `cd` into the project and deactivated when you leave — no manual `source .venv/bin/activate` needed.

---

## Team Defaults

If your team keeps a shared `vex-config.toml`, you can layer it in explicitly:

```bash
vex sync --from https://company.example/vex-config.toml
```

Local `.tool-versions` entries still win over matching tools from the team baseline, so repo-specific pins stay explicit.

---

## Working with Projects

### Pin a version for your project

```bash
cd my-project
vex local node@20.11.0
```

This creates a `.tool-versions` file:

```
node 20.11.0
```

### Install all project dependencies

When you clone a project with a `.tool-versions` file:

```bash
cd cloned-project
vex install
```

This installs all tools listed in `.tool-versions`.

### Auto-switching

With shell integration enabled, vex automatically switches versions when you `cd`:

```bash
cd project-a  # Uses node 20.11.0
cd project-b  # Uses node 22.0.0
```

## Supported Tools

| Tool | Example | Notes |
|------|---------|-------|
| Node.js | `vex install node@20` | Includes npm, npx (+ corepack in v24 and earlier) |
| Go | `vex install go@1.23` | Official Go binaries |
| Java | `vex install java@21` | Eclipse Temurin JDK |
| Rust | `vex install rust@stable` | Complete toolchain (rustc, cargo, clippy, etc.) |

## Version Aliases

vex supports fuzzy version matching and aliases:

```bash
# Node.js
vex install node@latest      # Latest version
vex install node@lts          # Latest LTS
vex install node@lts-iron     # Specific LTS codename
vex install node@20           # Latest 20.x

# Go
vex install go@latest         # Latest version
vex install go@1.23           # Latest 1.23.x

# Java
vex install java@latest       # Latest version
vex install java@lts          # Latest LTS

# Rust
vex install rust@stable       # Latest stable
vex install rust@latest       # Same as stable
```

Inspect remote versions before choosing a built-in alias:

```bash
vex list-remote node --filter lts
```

## Next Steps

- [Installation Guide](installation.md) - Detailed installation options
- [Command Reference](command-reference.md) - Full CLI reference
- [Shell Integration Guide](shell-integration.md) - Advanced shell setup
- [Troubleshooting Guide](troubleshooting.md) - Common issues and solutions
- [Main README](../../README.md) - Full feature list and documentation

## Common Workflows

### Upgrade to latest version

```bash
vex upgrade node
```

This installs the latest version and switches to it.

### Use different versions in different terminals

vex uses symlinks, so all terminals share the same active version. To use different versions:

1. Use Docker containers
2. Use separate user accounts
3. Manually set PATH in each terminal

### Global default version

Set a global default that applies everywhere unless overridden by a project `.tool-versions`:

```bash
vex global node@20.11.0
```

This writes to `~/.vex/tool-versions` (not `~/.tool-versions`), keeping all vex data under `~/.vex/`.

## Tips

- **Check before installing**: Use `vex list-remote <tool>` to see available versions
- **Use fuzzy matching**: `node@20` is easier than `node@20.11.0`
- **Run vex doctor**: If something isn't working, `vex doctor` can diagnose it
- **Keep versions clean**: Uninstall old versions you don't need anymore

## Getting Help

- Run `vex --help` for command help
- Run `vex <command> --help` for command-specific help
- Check the [Troubleshooting Guide](troubleshooting.md)
- File an issue on [GitHub](https://github.com/imnotnoahhh/vex/issues)

Happy version managing! 🚀
