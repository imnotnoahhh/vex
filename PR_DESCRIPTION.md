# Build Rich Terminal UI Foundation

Closes #57

## Summary

This PR implements a shared terminal UI layer for vex, providing consistent rendering primitives across all commands.

## Changes

### New Module: `src/ui.rs`

Created comprehensive UI components:
- **UiContext**: Detects interactive vs non-interactive mode
- **Basic Functions**: `header()`, `success()`, `warning()`, `error()`, `info()`, `dimmed()`
- **Table**: Builder for aligned tabular output
- **Progress**: Spinner for indeterminate operations
- **ProgressBar**: Progress bar for known-total operations
- **Summary**: Builder for final status summaries
- **Prompts**: `confirm()`, `select()`, `input()`

### Updated Commands

- **installer**: Uses `Progress` for installation steps
- **current**: Uses `Table` for tool/version display
- **outdated**: Uses `Table` for outdated tools
- **upgrade**: Uses `Summary` for upgrade results
- **doctor**: Uses UI primitives for check results

### Dependencies

Added `atty = "0.2"` for terminal detection.

### Tests

Added `tests/ui_test.rs` with comprehensive coverage of all UI components.

## Design Principles

1. **Separation of Concerns**: Data collection separate from rendering
2. **Non-Interactive Support**: All components work in piped/non-TTY environments
3. **JSON Compatibility**: JSON output paths unchanged
4. **Consistent Styling**: Unified colors and symbols
5. **Builder Pattern**: Fluent API for complex components

## Testing

Basic verification passed:
```bash
./verify-ui.sh
```

Full test suite requires:
```bash
cargo test --all-features -- --test-threads=1
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo build --release
```

## Acceptance Criteria

- ✅ Shared rendering primitives created
- ✅ `install`, `current`, `outdated`, `doctor` commands updated
- ✅ Non-interactive mode supported
- ✅ JSON output unchanged
- ✅ Tests added

## Screenshots

(To be added after manual testing in environment with Rust toolchain)

## Notes

Implementation completed in worktree. Ready for review after full test suite passes.
