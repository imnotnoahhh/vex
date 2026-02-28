# Contributing to vex

Thank you for your interest in vex. We welcome issues and pull requests from the community.

## Development Environment

**Requirements:**
- Rust stable (recommended via [rustup](https://rustup.rs/))
- macOS (Apple Silicon or Intel)

```bash
git clone https://github.com/imnotnoahhh/vex.git
cd vex
cargo build
```

## Pre-PR Checks

Before opening a PR, make sure all checks pass:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

CI runs fmt, clippy, test, and audit checks.

## Documentation Organization

- Keep stable, public project docs in the repository root (for example: `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`).
- Put process or archival docs under `docs/archive/`.
- `docs/archive/` should generally stay ignored in `.gitignore` to avoid committing temporary notes.

## Project Structure

```
src/
├── main.rs          # CLI entry, command routing, local/global/install flows
├── tools/           # Tool adapters (download URL, remote versions, fuzzy matching)
│   ├── mod.rs       # Tool trait, get_tool(), resolve_fuzzy_version()
│   ├── node.rs
│   ├── go.rs
│   ├── java.rs
│   └── rust.rs
├── downloader.rs    # HTTP download, SHA256 verification, retry logic
├── installer.rs     # Extract tar.gz and create version directory
├── switcher.rs      # Symlink management for bin/ and current/
├── resolver.rs      # Version file resolution (.tool-versions / .node-version)
├── shell.rs         # zsh/bash hook generation
└── error.rs         # Unified error types
```

## Adding Support for a New Tool

1. Add a new module under `src/tools/` (for example, `python.rs`).
2. Implement the `Tool` trait (`name`, `list_remote`, `download_url`, `bin_names`, `bin_subpath`).
3. Register it in `get_tool()` in `src/tools/mod.rs`.
4. Add unit tests and CLI integration tests.

## Commit and PR Guidelines

### Branch Strategy

- `main` — production branch, all PRs merge here
- Feature branches follow conventional commit types:
  - `feat/xxx` — new features
  - `fix/xxx` — bug fixes
  - `docs/xxx` — documentation changes
  - `chore/xxx` — maintenance tasks (dependencies, configs)
  - `ci/xxx` — CI/CD changes

### Conventional Commits

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>

[optional body]
```

**Types:**
- `feat:` — new feature
- `fix:` — bug fix
- `docs:` — documentation only
- `chore:` — maintenance (deps, configs, etc.)
- `ci:` — CI/CD changes
- `deps:` — dependency updates

**Examples:**
```
feat: add Python support
fix: resolve Node.js download checksum mismatch
docs: clarify installation steps in README
chore: bump MSRV to 1.89
```

### Pull Request Process

1. Create a branch from `main` with the appropriate prefix (`feat/`, `fix/`, etc.)
2. Make your changes and commit with conventional commit messages
3. Open a PR to `main`
4. Ensure all CI checks pass
5. Wait for review and approval

## Testing

- Put unit tests in each module under `#[cfg(test)] mod tests`.
- CLI integration tests live in `tests/cli_test.rs`.
- Mark network-dependent tests with `#[ignore]` so CI does not run them by default.
