# Testing Guide

This document describes the testing strategy and guidelines for vex.

## Table of Contents

- [Test Organization](#test-organization)
- [Running Tests](#running-tests)
- [Test Types](#test-types)
- [Writing Tests](#writing-tests)
- [Test Coverage](#test-coverage)
- [CI/CD Testing](#cicd-testing)

## Test Organization

vex uses a multi-layered testing approach:

```
vex/
├── src/
│   ├── main.rs              # Unit tests: #[cfg(test)] mod tests
│   ├── tools/
│   │   ├── mod.rs           # Unit tests for Tool trait
│   │   ├── node.rs          # Unit tests for Node.js adapter
│   │   ├── go.rs            # Unit tests for Go adapter
│   │   ├── java.rs          # Unit tests for Java adapter
│   │   └── rust.rs          # Unit tests for Rust adapter
│   ├── downloader.rs        # Unit tests for HTTP download
│   ├── installer.rs         # Unit tests for installation logic
│   ├── switcher.rs          # Unit tests for symlink management
│   ├── resolver.rs          # Unit tests for version file parsing
│   ├── shell.rs             # Unit tests for shell hooks
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
cargo test --all-features
```

This runs all unit tests and CLI integration tests, but skips tests marked with `#[ignore]`.

### Network-Dependent Tests

```bash
cargo test --all-features -- --ignored
```

These tests require internet access and may be slow. They are skipped in CI.

### All Tests (Including Network-Dependent)

```bash
cargo test --all-features -- --include-ignored
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
- `vex env zsh` generates correct hook
- `vex current` shows active versions
- `vex doctor` validates installation

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
- Mark with `#[ignore]` (network-dependent)
- Use real API endpoints
- Test with small, fast downloads when possible
- Clean up installed versions after tests

**Example**:
```rust
#[test]
#[ignore] // Network-dependent
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

Mark tests that require network access with `#[ignore]`:

```rust
#[test]
#[ignore] // Network-dependent
fn test_download_from_api() {
    // Test code that makes HTTP requests
}
```

Run these tests explicitly:
```bash
cargo test -- --ignored
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
#[ignore] // Network-dependent
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

### Current Coverage

As of v0.1.6:
- **Overall**: 66.51%
- **Lines covered**: 828/1245
- **Unit tests**: 133 tests
- **CLI integration tests**: 43 tests
- **E2E tests**: 5 tests
- **Total**: 181 tests

### 100% Coverage Modules

- `cache.rs`
- `resolver.rs`
- `shell.rs`
- `switcher.rs`
- `tools/mod.rs`

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
    - run: cargo test --all-features
```

### What CI Tests

- ✅ Unit tests
- ✅ CLI integration tests
- ✅ Code formatting (`cargo fmt`)
- ✅ Linting (`cargo clippy`)
- ✅ Security audit (`cargo audit`)
- ❌ Network-dependent tests (skipped)
- ❌ Benchmarks (skipped)

### Local Pre-Push Checks

Before pushing, run:

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Optional: Run ignored tests
cargo test --all-features -- --ignored
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
- ✅ Mark network tests with `#[ignore]`
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

- Check for network dependencies (mark with `#[ignore]`)
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
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
