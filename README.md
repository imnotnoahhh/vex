<h1 align="center">vex</h1>

<p align="center">
  <strong>A fast, multi-language version manager for macOS</strong>
</p>

<p align="center">
  Symlink-based switching · Node.js / Go / Java / Rust · .tool-versions · Auto-switch on cd
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
- **Multi-language** — manage Node.js, Go, Java (Eclipse Temurin), Rust from one tool
- **Fuzzy version matching** — `node@20` resolves to latest 20.x, `node@lts` to latest LTS
- **`.tool-versions` support** — per-project pinning, auto-switch on `cd`, batch install
- **Interactive selection** — `vex install node` lets you pick from a version list
- **Checksum verification** — Node.js uses official SHA256 verification; Go/Java/Rust follow upstream checksum metadata availability
- **macOS native** — supports both Apple Silicon and Intel macOS environments

## Quick Start

### Install

#### One-line installer (Recommended)

Automatically downloads the correct prebuilt binary for your macOS architecture (`arm64`/`x86_64`), installs to `~/.cargo/bin/vex`, and updates shell PATH in `~/.zshrc`, `~/.bashrc`, and `~/.bash_profile`:

```bash
# Latest release
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash

# Specific tag
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash -s -- --version v0.1.0
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
mkdir -p ~/.cargo/bin
cp vex-*/vex ~/.cargo/bin/vex
chmod +x ~/.cargo/bin/vex

# Add to PATH if not already present
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

#### Build from source

```bash
git clone https://github.com/imnotnoahhh/vex.git
cd vex
cargo build --release && cp target/release/vex ~/.cargo/bin/vex
```

Verify installation:

```bash
vex --version
```

### Setup

```bash
vex init

# Add shell hook to ~/.zshrc (auto-switch on cd)
echo 'eval "$(vex env zsh)"' >> ~/.zshrc
source ~/.zshrc
```

### Usage

```bash
# Interactive install (pick from version list)
vex install node

# Install a specific version (fuzzy matching)
vex install node@20          # → latest 20.x
vex install node@lts         # → latest LTS
vex install node@20.11.0     # → exact version

# Switch versions
vex use node@22

# Pin version for current project
vex local node@20.11.0       # writes .tool-versions

# Install everything from .tool-versions
vex install
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `vex init` | Initialize directory structure | `vex init` |
| `vex install <tool>` | Interactive install | `vex install node` |
| `vex install <tool@version>` | Install specific version | `vex install node@20` |
| `vex install` | Install all from `.tool-versions` | `vex install` |
| `vex use <tool@version>` | Switch to installed version | `vex use node@22` |
| `vex use --auto` | Auto-switch from version files | `vex use --auto` |
| `vex local <tool@version>` | Pin version in `.tool-versions` | `vex local node@20.11.0` |
| `vex global <tool@version>` | Pin version in `~/.tool-versions` | `vex global go@1.23` |
| `vex list <tool>` | List installed versions | `vex list node` |
| `vex list-remote <tool>` | List remote versions (interactive, latest 20) | `vex list-remote node` |
| `vex list-remote <tool> --all` | List all remote versions | `vex list-remote node --all` |
| `vex current` | Show active versions | `vex current` |
| `vex uninstall <tool@version>` | Uninstall a version | `vex uninstall node@20.11.0` |
| `vex env <shell>` | Output shell hook script | `vex env zsh` |

## Supported Tools

| Tool | Binaries | Source |
|------|----------|--------|
| Node.js | node, npm, npx | Official binaries |
| Go | go, gofmt | Official binaries |
| Java | java, javac, jar | Eclipse Temurin JDK |
| Rust | rustc, cargo | Official stable binaries |

## Fuzzy Version Matching

Version specs don't need to be exact:

```bash
vex install node@20       # latest 20.x.x
vex install node@20.11    # latest 20.11.x
vex install node@lts      # latest LTS release
vex install node@20.11.0  # exact version
vex install java@21       # exact (Java uses single numbers)
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
| Multi-language | ✅ | ❌ | ❌ | ✅ | ✅ |
| .tool-versions | ✅ | ❌ | ❌ | ✅ | ✅ |
| Auto-switch | ✅ | ❌ | ✅ | ✅ | ✅ |
| No shims | ✅ | ✅ | ✅ | ❌ | ❌ |
| Implementation | Rust | Shell | Rust | Shell | Rust |

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
├── cache/          # Download cache
└── config.toml
```

## FAQ

**Why does `vex list-remote go` not show every historical Go release?**
Go remote listings are constrained by upstream API policy and usually focus on active maintenance lines.

**Why does `vex list-remote rust` show limited choices?**
Current Rust support reads `channel-rust-stable.toml` and only targets the stable channel's current release.

**Why no `update` command?**
vex is a version manager, not a package manager. Version numbers are immutable. To use a newer version, just install it: `vex install node@22`.

**Can it coexist with nvm/fnm?**
Technically yes, but not recommended — PATH conflicts are likely.

**Windows/Linux support?**
macOS only for now.

**How to uninstall vex?**

```bash
# Remove shell hook from ~/.zshrc
# Then:
rm -rf ~/.vex
```

## Development

```bash
git clone https://github.com/imnotnoahhh/vex.git
cd vex
cargo build
cargo test --all-features
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

[MIT](LICENSE) © 2026 Noah Qin

## Acknowledgements

Inspired by [nvm](https://github.com/nvm-sh/nvm), [fnm](https://github.com/Schniz/fnm), [asdf](https://github.com/asdf-vm/asdf), and [mise](https://github.com/jdx/mise).
