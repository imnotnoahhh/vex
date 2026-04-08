# Testing Guide

This document describes the testing strategy and guidelines for vex.

## Table of Contents

- [Test Organization](#test-organization)
- [Running Tests](#running-tests)
- [Test Types](#test-types)
- [Writing Tests](#writing-tests)
- [Test Coverage](#test-coverage)
- [CI/CD Testing](#cicd-testing)
- [Failure-Recovery Tests](#6-failure-recovery-tests)

## Test Organization

vex uses a multi-layered testing approach:

```
vex/
├── src/
│   ├── main.rs              # Thin binary entry point
│   ├── main_tests.rs        # Top-level smoke-style unit tests
│   ├── app.rs / cli/        # CLI dispatch and argument definitions
│   ├── tools/
│   │   ├── tests.rs         # Tool trait and shared resolution tests
│   │   ├── node/tests.rs    # Node.js adapter tests
│   │   ├── go/tests.rs      # Go adapter tests
│   │   ├── java/tests.rs    # Java adapter tests
│   │   ├── rust/tests.rs    # Rust adapter tests
│   │   └── python/tests.rs  # Python adapter tests
│   ├── downloader.rs        # Checksum entrypoints and public API
│   ├── downloader/tests.rs  # Download transport tests
│   ├── installer/tests.rs   # Installation logic and cleanup tests
│   ├── switcher/tests.rs    # Symlink management and rollback tests
│   ├── resolver/tests.rs    # Version file parsing and discovery tests
│   ├── templates/tests.rs   # Project template rendering and merge-rule tests
│   ├── team_config/tests.rs # Safe remote/local team config loading tests
│   ├── shell/tests.rs       # Shell hook tests
│   ├── cache.rs             # Unit tests for caching
│   ├── lock.rs              # Unit tests for locking
│   └── error.rs             # Unit tests for error handling
├── tests/
│   ├── cli_test.rs          # CLI integration tests
│   └── e2e_test.rs          # End-to-end tests
└── benches/
    └── benchmarks.rs        # Performance benchmarks
```

## Running Tests

### All Tests (Excluding Network-Dependent)

```bash
cargo test
```

This runs all unit tests and CLI integration tests. Network-dependent Rust `cargo test` coverage still stays opt-in behind the `network-tests` feature.

### Network-Dependent Tests

```bash
cargo test --features network-tests
```

These tests require internet access and may be slow. They are opt-in locally. CI does not enable `cargo test --features network-tests`, but it does run dedicated live-network smoke scripts where release behavior matters.

### All Tests (Including Network-Dependent)

```bash
cargo test --features network-tests
```

### Specific Test

```bash
cargo test test_name
```

### Specific Module

```bash
cargo test --test cli_test
cargo test --test e2e_test
```

### With Output

```bash
cargo test -- --nocapture
```

### Benchmarks

```bash
cargo bench
```

Benchmarks are not run in CI to keep build times fast.

## Test Types

### 1. Unit Tests

**Location**: `#[cfg(test)] mod tests` in each source file

**Purpose**: Test individual functions and modules in isolation

**Examples**:
- Parse version strings
- Validate path components
- Check disk space calculations
- Verify checksum algorithms
- Verify template conflict handling and `add-only` merges
- Validate safe team config schema and local override precedence

**Guidelines**:
- Test both success and error cases
- Use `tempfile` for temporary directories
- Mock external dependencies when possible
- Keep tests fast (<100ms each)

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let version = parse_version("20.11.0").unwrap();
        assert_eq!(version, (20, 11, 0));
    }

    #[test]
    fn test_parse_version_invalid() {
        let result = parse_version("invalid");
        assert!(result.is_err());
    }
}
```

### 2. CLI Integration Tests

**Location**: `tests/cli_test.rs`

**Purpose**: Test CLI commands end-to-end without network access

**Examples**:
- `vex init` creates directory structure
- `vex init --template ...` previews or writes starter files
- `vex env zsh` generates correct hook
- `vex current` shows active versions
- `vex doctor` validates installation
- `vex install --from vex-config.toml` honors local `.tool-versions` overrides

**Guidelines**:
- Use `tempfile::TempDir` for isolated test environments
- Set `HOME` environment variable to temp directory
- Clean up after tests
- Test both success and error cases

**Example**:
```rust
#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("HOME", temp_dir.path());

    let output = Command::new("cargo")
        .args(&["run", "--", "init"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(temp_dir.path().join(".vex").exists());
}
```

### 3. End-to-End Tests

**Location**: `tests/e2e_test.rs`

**Purpose**: Test complete workflows with real network requests

**Examples**:
- Install Node.js from nodejs.org
- Switch between versions
- Auto-switch with `.tool-versions`
- Uninstall versions

**Guidelines**:
- Mark with `#[cfg_attr(not(feature = "network-tests"), ignore = "requires --features network-tests")]`
- Use real API endpoints
- Test with small, fast downloads when possible
- Clean up installed versions after tests

**Example**:
```rust
#[test]
#[cfg_attr(not(feature = "network-tests"), ignore = "requires --features network-tests")]
fn test_install_node() {
    let temp_dir = TempDir::new().unwrap();
    env::set_var("HOME", temp_dir.path());

    // Initialize vex
    Command::new("cargo")
        .args(&["run", "--", "init"])
        .output()
        .unwrap();

    // Install Node.js
    let output = Command::new("cargo")
        .args(&["run", "--", "install", "node@20.11.0"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(temp_dir.path()
        .join(".vex/toolchains/node/20.11.0")
        .exists());
}
```

### 4. Benchmarks

**Location**: `benches/benchmarks.rs`

**Purpose**: Measure performance of critical operations

**Examples**:
- Version file parsing
- Directory traversal
- Symlink creation
- Cache read/write
- Parallel vs sequential file extraction

**Guidelines**:
- Use `criterion` crate
- Benchmark realistic scenarios
- Include setup/teardown in measurements
- Run locally, not in CI

**Example**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_tool_versions(c: &mut Criterion) {
    let content = "node 20.11.0\ngo 1.23.5\njava 21\n";
    c.bench_function("parse_tool_versions", |b| {
        b.iter(|| parse_tool_versions(black_box(content)))
    });
}

criterion_group!(benches, bench_parse_tool_versions);
criterion_main!(benches);
```

### 5. Parallel Operations Tests

**Location**: Unit tests in `src/downloader/tests.rs`, `src/archive_cache/tests.rs`, and `src/installer/tests.rs`

**Purpose**: Test parallel download and extraction functionality

**Examples**:
- Atomic write with UUID-based temp files
- Parallel file extraction
- Error collection from parallel operations
- Cleanup of temporary files on failure

**Guidelines**:
- Test both parallel and sequential paths
- Verify atomic operations (temp file + rename)
- Test error handling in parallel contexts
- Ensure proper cleanup of temporary files

**Example**:
```rust
#[test]
fn test_atomic_write_cleanup_on_error() {
    let dir = std::env::temp_dir().join("vex_test_atomic_cleanup");
    std::fs::create_dir_all(&dir).unwrap();
    let dest = dir.join("test.txt");

    // Try to download from invalid URL
    let result = download_file("http://invalid.url/file", &dest);
    assert!(result.is_err());

    // Verify no temp files left behind
    let entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 0, "Temp files should be cleaned up");
}
```

### 6. Failure-Recovery Tests

**Location**: Unit tests in `src/installer/tests.rs` and `src/switcher/tests.rs`, plus CLI coverage in `tests/cli_test.rs`

**Purpose**: Prove that partial failures leave `~/.vex` in a deterministic and recoverable state

**Current examples**:
- post-install failures clean up the partially moved final toolchain directory
- switch failures roll back to the previously active version
- template conflict handling avoids partial writes
- safe team-config parsing rejects unsupported fields before touching install/sync flows

**Guidelines**:
- Prefer deterministic failure fixtures over timing-sensitive tests
- Keep any injected failure points test-only (`#[cfg(test)]`)
- Assert on cleanup and rollback state, not just the top-level error message
- Cover both the temp path and the final on-disk path when testing installer cleanup

## Writing Tests

### Test Naming Conventions

- Unit tests: `test_<function_name>_<scenario>`
- Integration tests: `test_<command>_<scenario>`
- E2E tests: `test_<workflow>_<scenario>`

**Examples**:
- `test_parse_version_valid`
- `test_parse_version_invalid`
- `test_install_command_success`
- `test_install_node_and_switch`

### Test Structure

Follow the **Arrange-Act-Assert** pattern:

```rust
#[test]
fn test_example() {
    // Arrange: Set up test data
    let input = "test input";
    let expected = "expected output";

    // Act: Execute the code under test
    let result = function_under_test(input);

    // Assert: Verify the result
    assert_eq!(result, expected);
}
```

### Using Temporary Directories

Always use `tempfile::TempDir` for file system tests:

```rust
use tempfile::TempDir;

#[test]
fn test_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Test code here

    // TempDir automatically cleans up when dropped
}
```

### Testing Error Cases

Test both success and error paths:

```rust
#[test]
fn test_function_success() {
    let result = function("valid input");
    assert!(result.is_ok());
}

#[test]
fn test_function_error() {
    let result = function("invalid input");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Expected error message"
    );
}
```

### Network-Dependent Tests

Mark tests that require network access with the `network-tests` opt-in:

```rust
#[test]
#[cfg_attr(not(feature = "network-tests"), ignore = "requires --features network-tests")]
fn test_download_from_api() {
    // Test code that makes HTTP requests
}
```

Run these tests explicitly:
```bash
cargo test --features network-tests
```

### Security Testing

When testing security features, be thorough:

#### Path Traversal Protection

```rust
#[test]
fn test_path_traversal_rejected() {
    let paths = vec![
        "../etc/passwd",
        "../../etc/passwd",
        "/etc/passwd",
        "foo/../../../etc/passwd",
    ];

    for path in paths {
        let result = validate_path(path);
        assert!(result.is_err(), "Path should be rejected: {}", path);
    }
}

#[test]
fn test_safe_paths_accepted() {
    let paths = vec![
        "foo/bar",
        "./foo/bar",
        "foo/./bar",
    ];

    for path in paths {
        let result = validate_path(path);
        assert!(result.is_ok(), "Path should be accepted: {}", path);
    }
}
```

#### Disk Space Check

```rust
#[test]
fn test_disk_space_sufficient() {
    let result = check_disk_space(1_000_000_000); // 1 GB
    assert!(result.is_ok());
}

#[test]
fn test_disk_space_insufficient() {
    let result = check_disk_space(1_000_000_000_000_000); // 1 PB
    assert!(result.is_err());
}
```

#### HTTP Timeout

```rust
#[test]
#[cfg_attr(not(feature = "network-tests"), ignore = "requires --features network-tests")]
fn test_http_timeout() {
    let client = create_http_client();
    let result = client.get("http://httpbin.org/delay/10").send();
    // Should timeout before 10 seconds
    assert!(result.is_err());
}
```

#### Checksum Verification

```rust
#[test]
fn test_checksum_valid() {
    let data = b"test data";
    let expected = "sha256_hash_here";
    let result = verify_checksum(data, expected);
    assert!(result.is_ok());
}

#[test]
fn test_checksum_invalid() {
    let data = b"test data";
    let expected = "wrong_hash";
    let result = verify_checksum(data, expected);
    assert!(result.is_err());
}
```

## Test Coverage

### Current Coverage Model

As of v1.6.1, validation is intentionally split across several layers:

- **Unit tests** in `src/**/*.rs` for parsing, resolution, downloading, switching, locking, and tool adapters
- **CLI integration tests** in `tests/cli_test.rs` for core command behavior without full external installs
- **End-to-end tests** in `tests/e2e_test.rs` for real installation workflows
- **Shell and feature smoke tests** in `scripts/test-features.sh`, `scripts/test-management-features.sh`, `scripts/test-shell-hooks.sh`, `scripts/test-rust-extensions-live.sh`, `scripts/test-security.sh`, and `scripts/test-performance.sh`
- **Strict macOS validation** in `scripts/test_vex_release_strict.py` for local builds by default, with published-release validation available through release-postflight or an explicit `workflow_dispatch` opt-in, covering official-archive diffs, multi-version switching, Python venv flows, project/global auto-switch behavior, and shell export hook coverage

### 100% Coverage Modules

- `cache.rs`
- `resolver.rs`
- `shell.rs`
- `switcher.rs`
- `tools/mod.rs`

### High Coverage Modules (>80%)

- `downloader.rs` - includes parallel download tests
- `installer.rs` - includes parallel extraction tests
- `error.rs`
- `lock.rs`

### Measuring Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --all-features --out Html --output-dir coverage

# Open report
open coverage/index.html
```

### Coverage Goals

- **Critical modules**: 90%+ (installer, downloader, switcher)
- **Core modules**: 80%+ (tools, resolver, cache)
- **Supporting modules**: 70%+ (shell, lock, error)

### Documentation-Driven Validation

For release readiness on macOS, the most important commands are:

```bash
bash scripts/test-features.sh
VEX_TEST_HOME=/tmp/vex-audit-home \
VEX_STRICT_TMP_ROOT=/tmp/strict-local-build \
VEX_STRICT_USE_LOCAL_BUILD=1 \
VEX_STRICT_VEX_BIN="$(pwd)/target/debug/vex" \
python3 scripts/test_vex_release_strict.py
python3 scripts/test_vex_release_strict.py
VEX_BIN="$(pwd)/target/debug/vex" bash scripts/test-rust-extensions-live.sh
```

The strict validation scripts cover:
- top-level CLI help and subcommand help
- `vex init`, `vex env`, and shell hook generation
- `vex repair migrate-home` and captured export refresh coverage
- fresh installs for Node.js, Go, Java, Rust, and Python
- Rust manifest parsing and official target/component command coverage through fixtures and CLI tests
- official archive binary diffing against local installs
- binary runnability and symlink correctness
- Python `.venv` init/freeze/sync workflows
- manual multi-version switching and project/global `cd` auto-switching

The Rust live smoke covers:
- official Rust toolchain installation from Rust upstream
- live download and installation of `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- live download and installation of `rust-src`
- metadata recording for managed Rust extensions
- managed cleanup for `vex rust target remove` and `vex rust component remove`

### Manual macOS Smoke Checklist

When you want a release-candidate sanity pass by hand, use an isolated `HOME` so you do not touch your real `~/.vex` state:

```bash
export VEX_BIN="$HOME/.local/bin/vex"
export VEX_TEST_HOME=/tmp/vex-manual-smoke
export VEX_TEST_REPO=/tmp/vex-manual-project

rm -rf "$VEX_TEST_HOME" "$VEX_TEST_REPO"
mkdir -p "$VEX_TEST_HOME" "$VEX_TEST_REPO"

HOME="$VEX_TEST_HOME" "$VEX_BIN" init
touch "$VEX_TEST_HOME/.zshrc"
printf 'export PATH="$HOME/.vex/bin:$PATH"\neval "$(vex env zsh)"\n' >> "$VEX_TEST_HOME/.zshrc"
export PATH="$VEX_TEST_HOME/.vex/bin:$PATH"
HOME="$VEX_TEST_HOME" "$VEX_BIN" doctor

HOME="$VEX_TEST_HOME" "$VEX_BIN" install node@20
HOME="$VEX_TEST_HOME" "$VEX_BIN" install go@latest
HOME="$VEX_TEST_HOME" "$VEX_BIN" install rust@stable
HOME="$VEX_TEST_HOME" "$VEX_BIN" install java@21
HOME="$VEX_TEST_HOME" "$VEX_BIN" install python@3.12
HOME="$VEX_TEST_HOME" "$VEX_BIN" use rust@stable
HOME="$VEX_TEST_HOME" "$VEX_BIN" rust target add aarch64-apple-ios aarch64-apple-ios-sim
HOME="$VEX_TEST_HOME" "$VEX_BIN" rust component add rust-src

HOME="$VEX_TEST_HOME" "$VEX_BIN" current
node -v
go version
rustc --version
java -version
python3 --version
```

Notes:
- Create a temporary shell rc file in the isolated home before `vex doctor` so shell-hook checks do not fail just because the temp home started empty.
- Export `PATH="$VEX_TEST_HOME/.vex/bin:$PATH"` before `vex doctor` so PATH checks reflect the isolated test environment.
- Use `go@latest` or an active minor from `vex list-remote go`; do not hardcode stale Go lines in release smoke steps.
- After the core tool installs pass, verify `vex rust target list` and `vex rust component list`, then continue with `.tool-versions`, `vex run`, `vex exec`, and `vex python init/freeze/sync` checks from the strict scripts if you want full manual coverage.

## CI/CD Testing

### GitHub Actions Workflow

```yaml
test:
  name: Test
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v6
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - run: cargo test
```

### What CI Tests

- ✅ Unit tests
- ✅ CLI integration tests
- ✅ Code formatting (`cargo fmt`)
- ✅ Linting (`cargo clippy`)
- ✅ Security audit (`cargo audit`)
- ✅ Dedicated live-network smoke for Rust official targets/components on macOS
- ❌ `cargo test --features network-tests` (still skipped by default)
- ❌ Benchmarks (skipped)

### Local Pre-Push Checks

Before pushing, run:

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Optional: Run network-dependent tests
cargo test --features network-tests

# Check release/homebrew tooling
bash scripts/check-release-tooling.sh
bash scripts/test-management-features.sh
VEX_BIN="$(pwd)/target/debug/vex" bash scripts/test-shell-hooks.sh
VEX_BIN="$(pwd)/target/debug/vex" bash scripts/test-rust-extensions-live.sh
```

Or use the Makefile:

```bash
make fmt
make clippy
make test
```

## Best Practices

### DO

- ✅ Write tests for new features
- ✅ Write tests for bug fixes
- ✅ Test both success and error cases
- ✅ Use descriptive test names
- ✅ Keep tests fast and isolated
- ✅ Clean up after tests (use `TempDir`)
- ✅ Gate network tests behind `network-tests`
- ✅ Test security features thoroughly

### DON'T

- ❌ Commit failing tests
- ❌ Skip tests in CI without good reason
- ❌ Write tests that depend on external state
- ❌ Write tests that depend on execution order
- ❌ Write tests that are flaky
- ❌ Write tests that take >1 second (unless E2E)
- ❌ Hardcode paths or environment variables

## Troubleshooting

### Tests Fail Locally But Pass in CI

- Check environment variables (`HOME`, `PATH`)
- Check file permissions
- Check for leftover test artifacts

### Tests Pass Locally But Fail in CI

- Check for network dependencies (gate with `network-tests`)
- Check for platform-specific code (macOS vs Linux)
- Check for timing issues (use `sleep` or retries)

### Flaky Tests

- Add retries for network operations
- Use longer timeouts
- Check for race conditions
- Ensure proper cleanup

### Slow Tests

- Profile with `cargo test -- --nocapture`
- Use smaller test data
- Mock expensive operations
- Move to E2E tests if necessary

## References

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion Benchmarking](https://github.com/bheisler/criterion.rs)
- [Tarpaulin Coverage](https://github.com/xd009642/tarpaulin)
- [../../CONTRIBUTING.md](../../CONTRIBUTING.md) - Contribution guidelines
