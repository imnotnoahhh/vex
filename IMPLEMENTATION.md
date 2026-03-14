# Issue #61 Implementation: Improve Version Resolution Errors

## Summary

Enhanced version resolution error messages to provide intelligent suggestions when a requested version is not found.

## Changes Made

### 1. Error Type Enhancement (`src/error.rs`)

Modified `VexError::VersionNotFound` to include a `suggestions` field:

```rust
#[error("Version not found: {tool}@{version}{suggestions}\n\nRun 'vex list-remote {tool}' to see all available versions.")]
VersionNotFound {
    tool: String,
    version: String,
    suggestions: String,  // NEW: Contains formatted suggestions
}
```

### 2. Suggestion Generation (`src/tools/mod.rs`)

Added `generate_version_suggestions()` function that provides:

1. **Same major version**: Latest version in the requested major line (e.g., 20.x)
2. **Same minor version**: Latest version in the requested minor line (e.g., 20.11.x)
3. **Nearby versions**: Versions within 2 major versions
4. **Latest overall**: The most recent version available

Example output:
```
Version not found: node@20.99.0

Did you mean:
  - 20.11.0 (latest in 20.x)
  - 21.0.0
  - 22.5.0 (latest)

Run 'vex list-remote node' to see all available versions.
```

### 3. Integration

Updated all 7 files that use `VexError::VersionNotFound`:
- `src/main.rs` - uninstall command
- `src/switcher.rs` - version switching
- `src/commands/updates.rs` - upgrade command
- `src/activation.rs` - version activation
- `src/tools/java.rs` - Java download URL
- `src/tools/python.rs` - Python download URL
- `src/error.rs` - tests

### 4. Tests Added

Added 5 new unit tests in `src/tools/mod.rs`:
- `test_generate_version_suggestions_same_major`
- `test_generate_version_suggestions_same_minor`
- `test_generate_version_suggestions_nearby`
- `test_generate_version_suggestions_latest`
- `test_generate_version_suggestions_empty`

Updated existing tests to handle the new error structure.

## Design Decisions

1. **No auto-correction**: Suggestions are informational only, never silently applied
2. **No extra network calls**: Uses already-fetched version data from `list_remote()`
3. **Tool-aware**: Suggestions respect tool-specific version formats
4. **Readable output**: Works in both interactive and non-interactive modes

## Acceptance Criteria

✅ Common invalid version inputs show useful suggestions
✅ Suggestions are tool-aware (respects major/minor version structure)
✅ Output stays readable in non-interactive mode
✅ Suggestion-ranking tests added
✅ Regression tests for exact, alias, and partial inputs maintained

## Files Modified

- `src/error.rs` - Error type definition
- `src/tools/mod.rs` - Suggestion generation logic
- `src/main.rs` - Error construction
- `src/switcher.rs` - Error construction
- `src/commands/updates.rs` - Error construction
- `src/activation.rs` - Error construction
- `src/tools/java.rs` - Error construction
- `src/tools/python.rs` - Error construction

## Testing

Run the following commands to verify:

```bash
cargo test --all-features -- --test-threads=1
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Or use the convenience script:

```bash
./verify.sh
```
