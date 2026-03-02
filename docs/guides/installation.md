# Installation Guide

This guide covers all methods of installing vex on macOS.

## System Requirements

- **Operating System**: macOS 11.0 (Big Sur) or later
- **Architecture**: Apple Silicon (M1/M2/M3) or Intel (x86_64)
- **Disk Space**:
  - vex binary: ~10 MB
  - Recommended: At least 1 GB for installing tools
  - Production use: 5-10 GB for multiple tools and versions
- **Network**: Internet connection for downloading binaries

## Installation Methods

### Method 1: One-Line Installer (Recommended)

The quickest way to install vex:

```bash
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash
```

This script will:
1. Detect your Mac architecture (Apple Silicon or Intel)
2. Download the latest release from GitHub
3. Extract and install to `~/.local/bin/vex`
4. Add `~/.local/bin` to your PATH (if not already present)

#### Install Specific Version

```bash
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash -s -- --version v0.1.6
```

#### Audit the Script First

For security, you can review the script before running:

```bash
curl -fsSL -o install-release.sh https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh
less install-release.sh
bash install-release.sh
```

### Method 2: Manual Download from GitHub Releases

1. Go to the [Releases page](https://github.com/imnotnoahhh/vex/releases)

2. Download the appropriate binary for your Mac:
   - **Apple Silicon** (M1/M2/M3): `vex-aarch64-apple-darwin.tar.gz`
   - **Intel**: `vex-x86_64-apple-darwin.tar.gz`

3. Extract the archive:
   ```bash
   tar -xzf vex-*.tar.gz
   ```

4. Move the binary to your PATH:
   ```bash
   mkdir -p ~/.local/bin
   mv vex-*/vex ~/.local/bin/vex
   chmod +x ~/.local/bin/vex
   ```

5. Add `~/.local/bin` to your PATH (if not already present):

   **For zsh** (default on macOS):
   ```bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

   **For bash**:
   ```bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

### Method 3: Build from Source

If you have Rust installed:

```bash
# Clone the repository
git clone https://github.com/imnotnoahhh/vex.git
cd vex

# Build release binary
cargo build --release

# Install to ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/vex ~/.local/bin/vex

# Add to PATH (if needed)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

#### Build Requirements

- Rust stable (install via [rustup](https://rustup.rs/))
- Xcode Command Line Tools: `xcode-select --install`

## Verify Installation

After installation, verify vex is working:

```bash
vex --version
# vex 0.1.6

which vex
# /Users/yourname/.local/bin/vex
```

## Initialize vex

Create the vex directory structure:

```bash
vex init
```

This creates:
```
~/.vex/
├── bin/          # Symlinks (will be added to PATH)
├── toolchains/   # Installed versions
├── current/      # Active version symlinks
├── cache/        # Download cache
├── locks/        # Installation locks
└── config.toml   # Configuration
```

## Set Up Shell Integration

For automatic version switching on `cd`, add vex to your shell:

### zsh (Default on macOS)

```bash
echo 'eval "$(vex env zsh)"' >> ~/.zshrc
source ~/.zshrc
```

### bash

```bash
echo 'eval "$(vex env bash)"' >> ~/.bashrc
source ~/.bashrc
```

### fish

```bash
echo 'vex env fish | source' >> ~/.config/fish/config.fish
```

Then restart your shell or run:
```bash
source ~/.config/fish/config.fish
```

### nushell

```bash
# Create vex hook file
vex env nu | save -f ~/.config/nushell/vex.nu

# Add to config
echo 'source ~/.config/nushell/vex.nu' >> ~/.config/nushell/config.nu
```

Then restart nushell.

## Verify Shell Integration

After setting up shell integration:

```bash
# Create a test directory with .tool-versions
mkdir -p /tmp/vex-test
cd /tmp/vex-test
echo "node 20.11.0" > .tool-versions

# Install the version
vex install node@20.11.0

# cd into the directory (should auto-switch)
cd /tmp/vex-test
# vex should automatically switch to node 20.11.0

# Verify
vex current
# node 20.11.0
```

## Configuration

vex stores configuration in `~/.vex/config.toml`.

### Default Configuration

```toml
# Cache TTL for remote version lists (in seconds)
cache_ttl_secs = 300  # 5 minutes
```

### Customization

Edit `~/.vex/config.toml` to change settings:

```toml
# Cache remote version lists for 1 hour
cache_ttl_secs = 3600

# Cache for 1 day
cache_ttl_secs = 86400

# Disable caching (always fetch fresh)
cache_ttl_secs = 0
```

## Troubleshooting Installation

### vex command not found

**Problem**: Shell can't find vex after installation.

**Solution**: Ensure `~/.local/bin` is in your PATH:

```bash
echo $PATH | grep -q "$HOME/.local/bin" && echo "✓ In PATH" || echo "✗ Not in PATH"
```

If not in PATH, add it:

```bash
# For zsh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# For bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Permission denied

**Problem**: `vex: permission denied` when running.

**Solution**: Make the binary executable:

```bash
chmod +x ~/.local/bin/vex
```

### Wrong architecture

**Problem**: `Bad CPU type in executable` error.

**Solution**: You downloaded the wrong binary. Check your Mac's architecture:

```bash
uname -m
# arm64 = Apple Silicon (M1/M2/M3)
# x86_64 = Intel
```

Download the correct binary:
- Apple Silicon: `vex-aarch64-apple-darwin.tar.gz`
- Intel: `vex-x86_64-apple-darwin.tar.gz`

### Shell integration not working

**Problem**: Auto-switching doesn't work when you `cd`.

**Solution**: Run `vex doctor` to diagnose:

```bash
vex doctor
```

Common issues:
- Shell hook not added to shell config
- Shell config not sourced
- Wrong shell detected

### Installation fails with network error

**Problem**: Download fails or times out.

**Solution**:
1. Check your internet connection
2. Try again (vex retries automatically)
3. Download manually from GitHub Releases
4. Check firewall settings

## Updating vex

To update to the latest version:

### Using the Installer

```bash
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash
```

The installer will overwrite the existing binary.

### Manual Update

1. Download the latest release
2. Replace `~/.local/bin/vex` with the new binary
3. Verify: `vex --version`

### From Source

```bash
cd vex
git pull origin main
cargo build --release
cp target/release/vex ~/.local/bin/vex
```

## Uninstalling vex

To completely remove vex:

### 1. Remove Shell Integration

Edit your shell config file and remove these lines:

**For zsh** (`~/.zshrc`):
```bash
eval "$(vex env zsh)"
export PATH="$HOME/.vex/bin:$PATH"
```

**For bash** (`~/.bashrc`):
```bash
eval "$(vex env bash)"
export PATH="$HOME/.vex/bin:$PATH"
```

**For fish** (`~/.config/fish/config.fish`):
```bash
vex env fish | source
```

**For nushell** (`~/.config/nushell/config.nu`):
```bash
source ~/.config/nushell/vex.nu
```

### 2. Remove vex Data and Binary

```bash
# Remove all installed versions and data
rm -rf ~/.vex

# Remove vex binary
rm -f ~/.local/bin/vex

# Remove nushell hook (if using nushell)
rm -f ~/.config/nushell/vex.nu
```

### 3. Restart Shell

```bash
# For zsh/bash
exec $SHELL

# For fish
exec fish

# For nushell
exec nu
```

## Next Steps

- [Getting Started Guide](getting-started.md) - Learn basic usage
- [Shell Integration Guide](shell-integration.md) - Advanced shell setup
- [Troubleshooting Guide](troubleshooting.md) - Common issues and solutions

## Getting Help

If you encounter issues during installation:

1. Run `vex doctor` to diagnose problems
2. Check the [Troubleshooting Guide](troubleshooting.md)
3. Search [GitHub Issues](https://github.com/imnotnoahhh/vex/issues)
4. File a new issue with:
   - Your macOS version
   - Your Mac architecture (Apple Silicon or Intel)
   - Installation method used
   - Error messages
