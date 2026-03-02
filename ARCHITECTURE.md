# vex Architecture

This document describes the architecture and design decisions of vex, a multi-language version manager for macOS.

## Table of Contents

- [Overview](#overview)
- [Core Principles](#core-principles)
- [System Architecture](#system-architecture)
- [Module Dependencies](#module-dependencies)
- [Data Flow](#data-flow)
- [File System Layout](#file-system-layout)
- [Key Design Decisions](#key-design-decisions)

## Overview

vex is a Rust-based version manager that uses **symlinks + PATH prepending** (not shims) to provide instant version switching for Node.js, Go, Java, and Rust on macOS.

**Key characteristics:**
- Zero runtime overhead (no shim layer)
- Atomic version switching via symlinks
- Official binary distributions only (no compilation)
- Per-project version pinning via `.tool-versions`
- Automatic version switching on directory change

## Core Principles

1. **Simplicity**: Symlinks are simple, transparent, and debuggable
2. **Speed**: No shim overhead, instant version switching
3. **Safety**: Atomic operations, checksum verification, path validation
4. **Compatibility**: Works with existing tools and workflows
5. **Official binaries only**: No custom builds, trust upstream sources

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│  (main.rs - clap command parsing and routing)               │
└────────────┬────────────────────────────────────────────────┘
             │
             ├──────────────────────────────────────────────────┐
             │                                                  │
┌────────────▼──────────┐  ┌──────────────┐  ┌───────────────▼┐
│   Command Handlers    │  │   Resolver   │  │  Shell Hooks   │
│  (install/use/list)   │  │ (.tool-vers) │  │ (zsh/bash/fish)│
└────────────┬──────────┘  └──────┬───────┘  └────────────────┘
             │                    │
             │                    │
┌────────────▼────────────────────▼──────────────────────────┐
│                      Core Services                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │Downloader│  │Installer │  │ Switcher │  │  Cache   │  │
│  │(HTTP+SHA)│  │(tar+disk)│  │(symlinks)│  │ (5 min)  │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                    Tool Adapters                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │ Node.js │  │   Go    │  │  Java   │  │  Rust   │       │
│  │(LTS API)│  │(dl JSON)│  │(Adoptium│  │(channel)│       │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘       │
└─────────────────────────────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│                  File System Layer                           │
│  ~/.vex/                                                     │
│  ├── bin/           (symlinks to current/*/bin/*)           │
│  ├── current/       (symlinks to toolchains/*/version/)     │
│  ├── toolchains/    (installed versions)                    │
│  ├── cache/         (downloads + remote version lists)      │
│  ├── locks/         (installation locks)                    │
│  └── config.toml    (user configuration)                    │
└─────────────────────────────────────────────────────────────┘
```

## Module Dependencies

### Dependency Graph

```
main.rs
  ├─> tools/mod.rs (Tool trait)
  │     ├─> tools/node.rs
  │     ├─> tools/go.rs
  │     ├─> tools/java.rs
  │     └─> tools/rust.rs
  ├─> installer.rs
  │     ├─> downloader.rs
  │     ├─> lock.rs
  │     └─> error.rs
  ├─> switcher.rs
  │     └─> error.rs
  ├─> resolver.rs
  │     └─> error.rs
  ├─> shell.rs
  ├─> cache.rs
  │     └─> error.rs
  └─> error.rs
```

### Module Responsibilities

| Module | Responsibility | Key Functions |
|--------|---------------|---------------|
| `main.rs` | CLI entry point, command routing | `run()`, `install_tool()`, `run_doctor()` |
| `tools/mod.rs` | Tool trait definition, architecture detection | `Tool` trait, `get_tool()`, `resolve_fuzzy_version()` |
| `tools/node.rs` | Node.js adapter (nodejs.org API) | `list_remote()`, `download_url()`, `resolve_alias()` |
| `tools/go.rs` | Go adapter (go.dev JSON API) | `list_remote()`, `download_url()` |
| `tools/java.rs` | Java adapter (Adoptium API) | `list_remote()`, `download_url()` |
| `tools/rust.rs` | Rust adapter (channel TOML) | `list_remote()`, `download_url()`, `post_install()` |
| `downloader.rs` | HTTP download, SHA256 verification | `download()`, `verify_checksum()` |
| `installer.rs` | Extract archives, disk space check | `install()`, `check_disk_space()` |
| `switcher.rs` | Atomic symlink updates | `switch_version()` |
| `resolver.rs` | Version file parsing | `resolve_versions()`, `resolve_version()` |
| `shell.rs` | Shell hook generation | `generate_hook()` |
| `cache.rs` | Remote version list caching | `get_cached_versions()`, `cache_versions()` |
| `lock.rs` | Installation locking | `InstallLock::acquire()` |
| `error.rs` | Unified error handling | `VexError` enum |

## Data Flow

### Installation Flow

```
User: vex install node@20
         │
         ▼
    Parse spec (node, 20)
         │
         ▼
    Resolve fuzzy version (20 → 20.11.0)
         │
         ▼
    Check if already installed
         │
         ▼
    Acquire installation lock
         │
         ▼
    Check disk space (≥500 MB)
         │
         ▼
    Download tar.gz to cache/
    (with progress bar, timeout, retry)
         │
         ▼
    Verify SHA256 checksum
         │
         ▼
    Extract to temp directory
    (with path traversal validation)
         │
         ▼
    Move to toolchains/node/20.11.0/
         │
         ▼
    Run post_install() hook
         │
         ▼
    Switch to new version (update symlinks)
         │
         ▼
    Release lock, cleanup cache
```

### Version Switching Flow

```
User: vex use node@22
         │
         ▼
    Parse spec (node, 22)
         │
         ▼
    Check if version installed
         │
         ▼
    Create temp symlink:
    current/node.tmp → toolchains/node/22.0.0/
         │
         ▼
    Atomic rename:
    current/node.tmp → current/node
         │
         ▼
    Update bin/ symlinks:
    bin/node → current/node/bin/node
    bin/npm → current/node/bin/npm
    bin/npx → current/node/bin/npx
```

### Auto-Switch Flow (Shell Hook)

```
User: cd my-project/
         │
         ▼
    Shell hook triggered (chpwd/PROMPT_COMMAND)
         │
         ▼
    Traverse up directory tree
         │
         ▼
    Find .tool-versions file
         │
         ▼
    Parse: node 20.11.0
         │
         ▼
    Check if version installed
         │
         ▼
    Switch to version (if installed)
    (silently skip if not installed)
```

## File System Layout

```
~/.vex/
├── bin/                          # Added to PATH
│   ├── node → ../current/node/bin/node
│   ├── npm → ../current/node/bin/npm
│   ├── go → ../current/go/bin/go
│   ├── java → ../current/java/bin/java
│   └── rustc → ../current/rust/bin/rustc
│
├── current/                      # Active version symlinks
│   ├── node → ../toolchains/node/20.11.0
│   ├── go → ../toolchains/go/1.23.5
│   ├── java → ../toolchains/java/21
│   └── rust → ../toolchains/rust/1.93.1
│
├── toolchains/                   # Installed versions
│   ├── node/
│   │   ├── 20.11.0/
│   │   │   └── bin/
│   │   │       ├── node
│   │   │       ├── npm
│   │   │       └── npx
│   │   └── 22.0.0/
│   ├── go/
│   │   └── 1.23.5/
│   │       └── bin/go
│   ├── java/
│   │   └── 21/
│   │       └── Contents/Home/bin/java
│   └── rust/
│       └── 1.93.1/
│           ├── rustc/bin/rustc
│           ├── cargo/bin/cargo
│           └── clippy-preview/bin/clippy-driver
│
├── cache/                        # Temporary downloads
│   ├── node-v20.11.0-darwin-arm64.tar.gz
│   └── remote_versions/          # Cached API responses (5 min TTL)
│       ├── node.json
│       ├── go.json
│       └── java.json
│
├── locks/                        # Installation locks
│   ├── node-20.11.0.lock
│   └── go-1.23.5.lock
│
└── config.toml                   # User configuration
    cache_ttl_secs = 300
```

## Key Design Decisions

### 1. Symlinks vs Shims

**Decision**: Use symlinks + PATH prepending

**Rationale**:
- **Performance**: Zero runtime overhead (shims add ~10ms per invocation)
- **Transparency**: Users can inspect symlinks with `ls -la`
- **Compatibility**: Works with all tools (IDEs, scripts, etc.)
- **Simplicity**: No custom shim logic to maintain

**Trade-offs**:
- Requires shell restart or `hash -r` after switching
- Symlinks visible in `~/.vex/bin/`

### 2. Official Binaries Only

**Decision**: Download official pre-built binaries, never compile from source

**Rationale**:
- **Speed**: Installation takes seconds, not minutes
- **Reliability**: Official binaries are tested by upstream
- **Security**: Trust upstream build infrastructure
- **Simplicity**: No build dependencies (gcc, make, etc.)

**Trade-offs**:
- Limited to platforms with official binaries (macOS only for now)
- Cannot customize build flags

### 3. Atomic Version Switching

**Decision**: Use temporary symlink + atomic rename

**Rationale**:
- **Safety**: No partial state (either old or new version, never broken)
- **Concurrency**: Multiple processes can switch safely
- **Rollback**: Old symlink remains until rename succeeds

**Implementation**:
```rust
// Create temp symlink
fs::symlink(&target, &temp_link)?;
// Atomic rename (POSIX guarantees atomicity)
fs::rename(&temp_link, &final_link)?;
```

### 4. Path Traversal Protection

**Decision**: Validate all archive paths before extraction (v0.1.6+)

**Rationale**:
- **Security**: Prevent zip-slip attacks
- **Safety**: Malicious archives cannot write outside `~/.vex/`

**Implementation**:
- Reject paths containing `..`
- Reject absolute paths
- Use `unpack_in()` for controlled extraction

### 5. HTTP Timeout Configuration

**Decision**: 30s connect timeout, 5min total timeout, 3 retries (v0.1.6+)

**Rationale**:
- **Reliability**: Prevent indefinite hangs on slow networks
- **User experience**: Large files (JDK) need longer timeouts
- **Security**: Mitigate resource exhaustion attacks

### 6. Disk Space Check

**Decision**: Require 500 MB free space before installation (v0.1.6+)

**Rationale**:
- **Safety**: Prevent partial installations on full disks
- **User experience**: Fail fast with clear error message
- **Security**: Mitigate disk space exhaustion DoS

### 7. Version File Priority

**Decision**: `.tool-versions` overrides language-specific files

**Rationale**:
- **Consistency**: Single source of truth for multi-language projects
- **Compatibility**: Matches asdf/mise behavior
- **Simplicity**: Clear precedence rules

**Priority order**:
1. `.tool-versions` (highest)
2. `.node-version` / `.nvmrc`
3. `.go-version`
4. `.java-version`
5. `.rust-toolchain`

### 8. Shell Hook Design

**Decision**: Generate shell-specific hooks, not universal script

**Rationale**:
- **Performance**: Native shell syntax is faster
- **Compatibility**: Each shell has different hook mechanisms
- **Reliability**: No cross-shell compatibility issues

**Supported shells**:
- zsh: `add-zsh-hook chpwd`
- bash: `PROMPT_COMMAND`
- fish: `--on-variable PWD`
- nushell: `pre_prompt` hooks

### 9. Caching Strategy

**Decision**: Cache remote version lists for 5 minutes (configurable)

**Rationale**:
- **Performance**: Avoid repeated API calls
- **Reliability**: Reduce dependency on upstream availability
- **Freshness**: 5 minutes is short enough for new releases

**Implementation**:
- Store in `~/.vex/cache/remote_versions/`
- Check mtime before using cache
- Configurable via `config.toml`

### 10. Error Handling

**Decision**: Actionable error messages with troubleshooting steps (v0.1.6+)

**Rationale**:
- **User experience**: Help users fix issues themselves
- **Support**: Reduce support burden
- **Education**: Teach users about common problems

**Example**:
```
Error: Disk space insufficient: need 1 GB, available 0.5 GB

Possible causes:
- Disk is full or nearly full
- Large files in trash or downloads

Solutions:
- Free up disk space by deleting unnecessary files
- Empty trash and clear downloads folder
- Use 'vex uninstall' to remove unused versions
- Check disk usage: df -h
```

## Future Considerations

### Python Support

**Challenges**:
- No official macOS binaries (python.org provides installers, not tarballs)
- Multiple distribution options (CPython, PyPy, Anaconda)
- Complex dependency management (pip, virtualenv)

**Potential approaches**:
1. python-build-standalone (pre-built binaries)
2. uv (Rust-based Python installer)
3. conda-forge (community binaries)
4. Local compilation (slow, requires build tools)

### Cross-Platform Support

**Challenges**:
- Linux: Different distros, glibc vs musl
- Windows: Different binary formats, no symlinks

**Potential approaches**:
- Linux: Detect distro, use appropriate binaries
- Windows: Use junction points or hardlinks

### Plugin System

**Challenges**:
- Maintain security and safety guarantees
- Ensure plugin quality and compatibility

**Potential approaches**:
- Rust-based plugins (compile-time safety)
- WASM plugins (sandboxed execution)
- External tool adapters (JSON protocol)

## References

- [CLAUDE.md](CLAUDE.md) - Development guidelines
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guide
- [SECURITY.md](SECURITY.md) - Security policy
- [TESTING.md](TESTING.md) - Testing guidelines
