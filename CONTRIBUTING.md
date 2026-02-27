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

- Keep each PR focused on one change.
- Write concise commit messages that explain what changed and why.
- If your PR fixes an issue, reference it in the PR description (for example: `Fixes #123`).

## Testing

- Put unit tests in each module under `#[cfg(test)] mod tests`.
- CLI integration tests live in `tests/cli_test.rs`.
- Mark network-dependent tests with `#[ignore]` so CI does not run them by default.
