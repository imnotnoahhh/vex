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
├── main.rs          # CLI entry, command routing, local/global/install flows, doctor command
├── tools/           # Tool adapters (download URL, remote versions, fuzzy matching)
│   ├── mod.rs       # Tool trait, get_tool(), resolve_fuzzy_version()
│   ├── node.rs      # Node.js with LTS support
│   ├── go.rs        # Go with minor version matching
│   ├── java.rs      # Java (Eclipse Temurin JDK)
│   └── rust.rs      # Rust with complete toolchain
├── downloader.rs    # HTTP download, SHA256 verification, retry logic, timeout configuration
├── installer.rs     # Extract tar.gz, disk space check, path traversal protection
├── switcher.rs      # Symlink management for bin/ and current/
├── resolver.rs      # Version file resolution (.tool-versions / .node-version / etc.)
├── shell.rs         # Shell hook generation (zsh, bash, fish, nushell)
├── cache.rs         # Remote version list caching with TTL
├── lock.rs          # Installation lock mechanism
└── error.rs         # Unified error types with actionable suggestions
```

## Adding Support for a New Tool

1. Add a new module under `src/tools/` (for example, `python.rs`).
2. Implement the `Tool` trait (`name`, `list_remote`, `download_url`, `bin_names`, `bin_subpath`).
3. Register it in `get_tool()` in `src/tools/mod.rs`.
4. Add unit tests and CLI integration tests.

## Commit and PR Guidelines

### Branch Strategy

**Permanent Branch:**
- `main` — production branch, protected, only accepts PRs
  - Every merge to `main` should be release-ready
  - Tagged with version numbers (e.g., `v0.1.1`)
  - CI runs on every push

**Temporary Branches:**
Feature branches are created from `main` and deleted after merging. Branch names must match conventional commit types:

- `feat/xxx` — new features (e.g., `feat/add-python-support`)
- `fix/xxx` — bug fixes (e.g., `fix/node-checksum-validation`)
- `docs/xxx` — documentation changes (e.g., `docs/update-installation-guide`)
- `chore/xxx` — maintenance tasks (e.g., `chore/bump-dependencies`)
- `ci/xxx` — CI/CD changes (e.g., `ci/add-coverage-report`)

**Workflow:**
1. Create a branch from `main`: `git checkout -b feat/your-feature`
2. Make changes and commit with conventional commit messages
3. Push and open a PR to `main`
4. After merge, the branch is automatically deleted

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

### Test Organization

- **Unit tests**: Put in each module under `#[cfg(test)] mod tests`
- **CLI integration tests**: Live in `tests/cli_test.rs`
- **End-to-end tests**: Live in `tests/e2e_test.rs`
- **Benchmarks**: Live in `benches/benchmarks.rs`

### Running Tests

```bash
# Run all tests (excluding ignored)
cargo test --all-features

# Run all tests including network-dependent ones
cargo test --all-features -- --ignored

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench
```

### Test Guidelines

- Mark network-dependent tests with `#[ignore]` so CI does not run them by default
- Add tests for new features and bug fixes
- Ensure tests are deterministic and don't depend on external state
- Use `tempfile` crate for temporary directories in tests
- Test both success and error cases

### Security Testing

When adding security features, ensure comprehensive test coverage:

- **Path traversal protection**: Test with malicious paths (`../`, absolute paths)
- **Disk space checks**: Test with insufficient space scenarios
- **HTTP timeouts**: Test timeout and retry logic
- **Checksum verification**: Test with corrupted downloads
- **Lock mechanism**: Test concurrent installation attempts

See `TESTING.md` for detailed testing guidelines.
