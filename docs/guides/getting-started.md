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
vex init
```

This creates the `~/.vex` directory structure where vex stores installed versions.

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
# Interactive install (pick from a list)
vex install node

# Or install a specific version
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

## Basic Commands

### Install a tool

```bash
# Interactive (pick from list)
vex install node

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
# Show latest 20 versions (interactive)
vex list-remote node

# Show all versions
vex list-remote node --filter all
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

Use `.vex.toml` plus `vex run` for repeatable project tasks:

```toml
[commands]
test = "cargo test --all-features"
lint = "cargo clippy --all-targets --all-features -- -D warnings"
```

```bash
vex run test
vex run lint
```

## Python Workflow

Python support uses [python-build-standalone](https://github.com/astral-sh/python-build-standalone) — prebuilt CPython binaries, no compilation needed.

### Step 1 — Install Python globally

```bash
vex install python@3.12   # or: python@latest, python@stable
vex global python@3.12    # set as global default
```

### Step 2 — Set up a project

```bash
cd my-project
vex python init
```

This creates `.venv` in the current directory using the active vex-managed Python, and records the version in `.tool-versions`.

### Step 3 — Install packages and lock them

```bash
source .venv/bin/activate   # or let the shell hook do it automatically on next cd
pip install requests flask
vex python freeze            # writes requirements.lock
```

### Step 4 — Commit

```bash
git add .tool-versions requirements.lock
git commit -m "pin python and dependencies"
```

### Step 5 — Restore on another machine

```bash
vex install python@3.12
cd my-project
vex python sync   # creates .venv if missing, then pip install -r requirements.lock
```

### Auto-activation

With the shell hook enabled (`eval "$(vex env zsh)"`), the `.venv` is automatically activated when you `cd` into the project and deactivated when you leave — no manual `source .venv/bin/activate` needed.

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

See available aliases:

```bash
vex alias node
```

## Next Steps

- [Installation Guide](installation.md) - Detailed installation options
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
