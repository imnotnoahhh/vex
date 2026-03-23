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

## Testing Your Build Without Affecting Your Installed vex

vex stores all its data under `~/.vex/`. If you have a stable vex installed, running your dev build directly would read and write the same directory, potentially corrupting your installed tool versions.

The fix is to override `HOME` for your dev build, pointing it at a throwaway directory:

```bash
# Build the dev binary
cargo build --release

# Set up a convenience alias using an absolute path (works from any directory)
alias vex-dev="HOME=/tmp/vex-dev $(pwd)/target/release/vex"

# Now use vex-dev freely — it reads/writes /tmp/vex-dev/.vex/ only
vex-dev init
vex-dev install python@3.12
vex-dev list python
vex-dev doctor

# Your real ~/.vex/ is completely untouched
vex list python   # still shows your stable install
```

`HOME=<path> <command>` is standard shell syntax that overrides an environment variable for a single command only. `/tmp` is cleaned up on reboot, so there's no permanent mess.

When you're done testing, clean up manually if needed:

```bash
rm -rf /tmp/vex-dev
```

## Pre-PR Checks

Before opening a PR, make sure all checks pass:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
bash scripts/check-docs.sh
```

CI runs fmt, clippy, test, and audit checks.

## Documentation Organization

- Keep stable, public project docs in the repository root (for example: `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`).
- Keep maintainer and process docs under `docs/development/`.
- Put temporary notes or archival docs under `docs/archive/`.
- `docs/archive/` should generally stay ignored in `.gitignore` to avoid committing temporary notes.

## Project Structure

```text
src/
├── main.rs          # Thin binary entry point
├── app.rs           # CLI dispatch
├── cli/             # clap argument definitions
├── commands/        # Command implementations
├── tools/           # Tool adapters plus shared resolution helpers
├── downloader/      # Download transport, retry, and progress plumbing
├── installer/       # Online/offline install orchestration and extraction helpers
├── switcher/        # Symlink updates, rollback, and failure fixtures
├── resolver/        # Version file discovery and parsing
├── templates/       # Built-in project starters, merge planning, rollback-safe writes
├── team_config/     # Safe `--from` sources (`vex-config.toml`, HTTPS, Git)
├── shell/           # Shell detection and generated hooks
├── updater/         # Self-update release selection, extraction, and repair helpers
├── version_files.rs # `.tool-versions` and single-value writers
├── checksum.rs      # Shared SHA256 helpers
├── versioning.rs    # Shared version normalization helpers
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
  - Tagged with version numbers (for example, `v1.1.1`)
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
cargo test

# Run all tests including network-dependent ones
cargo test --features network-tests

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
- If you touch install/switch failure paths, add or update cleanup/rollback coverage
- If you change templates or team config behavior, update both CLI tests and docs in the same PR

### Security Testing

When adding security features, ensure comprehensive test coverage:

- **Path traversal protection**: Test with malicious paths (`../`, absolute paths)
- **Disk space checks**: Test with insufficient space scenarios
- **HTTP timeouts**: Test timeout and retry logic
- **Checksum verification**: Test with corrupted downloads
- **Lock mechanism**: Test concurrent installation attempts

See `docs/development/testing.md` for detailed testing guidelines.
