# Logging Framework

vex uses the `tracing` crate for structured logging, providing better debugging and performance analysis capabilities.

## Usage

### Enabling Logs

By default, vex logs at the `info` level. You can control the log level using the `VEX_LOG` environment variable:

```bash
# Show all logs (most verbose)
VEX_LOG=trace vex install node@20

# Show debug information
VEX_LOG=debug vex install node@20

# Show general information (default)
VEX_LOG=info vex install node@20

# Show warnings only
VEX_LOG=warn vex install node@20

# Show errors only
VEX_LOG=error vex install node@20
```

### Log Output

Logs include:
- Timestamp
- Log level (TRACE, DEBUG, INFO, WARN, ERROR)
- Source file and line number
- Structured message

Example output:
```
2026-03-11T10:30:45.123Z  INFO vex::installer: Starting installation: node@20.11.0 installer.rs:117
2026-03-11T10:30:45.124Z DEBUG vex::installer: Detected architecture: X64 installer.rs:118
2026-03-11T10:30:45.125Z DEBUG vex::installer: Acquiring install lock for node@20.11.0 installer.rs:136
2026-03-11T10:30:45.200Z  INFO vex::downloader: Starting download: https://nodejs.org/dist/v20.11.0/node-v20.11.0-darwin-x64.tar.gz downloader.rs:63
```

## Implementation Details

### Modules with Logging

The following modules have integrated logging:

1. **downloader.rs** - Download operations, retries, and errors
2. **installer.rs** - Installation process, disk space checks, lock acquisition
3. **switcher.rs** - Version switching operations

### Log Levels

- **TRACE**: Very detailed information, typically only useful for diagnosing problems
- **DEBUG**: Detailed information useful for debugging
- **INFO**: General informational messages about normal operation
- **WARN**: Warning messages about potentially problematic situations
- **ERROR**: Error messages about failures

### Adding Logs to New Code

When adding new functionality, use the appropriate log level:

```rust
use tracing::{debug, error, info, warn};

// General operation
info!("Starting operation: {}", operation_name);

// Detailed debugging
debug!("Processing item: {:?}", item);

// Warnings
warn!("Potential issue detected: {}", issue);

// Errors
error!("Operation failed: {}", error);
```

## Benefits

1. **Structured Logging**: Logs are structured and can be easily parsed
2. **Performance**: Minimal overhead when logging is disabled
3. **Debugging**: File and line numbers help locate issues quickly
4. **Flexibility**: Easy to filter logs by level or module

## Future Enhancements

Potential improvements:
- JSON output format for log aggregation
- Log rotation for long-running operations
- Performance tracing with spans
- Integration with external logging services
