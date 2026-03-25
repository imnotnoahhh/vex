# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in vex, please report it responsibly.

**Please do not open a public GitHub issue for security reports.**

Instead, email: **qinfuyaoo@icloud.com**

Please include:
- A clear description of the vulnerability
- Reproduction steps or proof of concept
- Affected versions
- Expected and observed impact

You should receive an initial response within 48 hours. We will work with you to validate and remediate the issue before any public disclosure.

## Scope

Security issues in scope include:

### Download and Installation Security
- **Checksum verification bypass**: SHA256 verification for downloaded binaries
- **Path traversal attacks**: Malicious tar archives attempting to write outside installation directory (e.g., `../../../etc/passwd`)
- **Arbitrary code execution**: Via crafted version strings or `.tool-versions` files
- **Supply chain risks**: In downloaded toolchain binaries from upstream sources

### Network Security
- **HTTP timeout vulnerabilities**: Indefinite hangs or resource exhaustion
- **Man-in-the-middle attacks**: During binary downloads (HTTPS enforcement)
- **Retry logic abuse**: Excessive retries causing DoS

### File System Security
- **Symlink traversal**: Malicious symlinks in `~/.vex/` directory structure
- **Disk space exhaustion**: Installations consuming all available disk space (DoS)
- **Permission issues**: Incorrect file permissions allowing unauthorized access

### Implemented Security Measures

vex includes the following security protections:

1. **TOCTOU Race Condition Protection** (v1.0.0+)
   - UUID v4-based temporary filenames prevent predictable paths
   - Directory ownership validation prevents privilege escalation
   - Atomic symlink operations with verification

2. **Atomic Write Protection** (v1.0.0+)
   - All downloads use UUID-based temporary files
   - Atomic rename operations prevent corruption
   - Automatic cleanup of temporary files on failure

3. **Path Traversal Protection** (v0.1.6+)
   - Validates all archive entry paths before extraction
   - Rejects paths containing `..` (parent directory references)
   - Rejects absolute paths
   - Prevents zip-slip style attacks

4. **HTTP Timeout Configuration** (v0.1.6+)
   - Connection timeout: 30 seconds
   - Total timeout: 5 minutes
   - Automatic retry: 3 attempts with exponential backoff
   - 4xx client errors (e.g., 404) are not retried
   - Prevents indefinite hangs and resource exhaustion

5. **Disk Space Check** (v0.1.6+)
   - Validates at least 500 MB free space before installation
   - Prevents partial installations on full disks
   - Mitigates disk space exhaustion DoS attacks

6. **Checksum Verification**
   - SHA256 verification for all downloads (Node.js, Python, Java, Rust)
   - Go checksums embedded in API response
   - Detects corrupted or tampered downloads

7. **Installation Locking** (v0.1.1+)
   - File-based locking prevents concurrent installation corruption
   - PID validation prevents deadlocks (v1.0.0+)
   - Automatic cleanup of stale locks

8. **Parallel Operation Safety** (v1.0.0+)
   - Parallel downloads limited to 3 concurrent operations
   - Parallel extraction with proper error handling
   - Resource exhaustion prevention

9. **Error Handling**
   - Actionable error messages with troubleshooting steps
   - No sensitive information leaked in error messages

10. **Safe Team Config Scope** (v1.5+)
   - Remote/shared team config is limited to `version = 1` plus `[tools]`
   - Team config cannot inject `env`, commands, mirrors, or arbitrary script execution
   - Local `.tool-versions` remains the highest-precedence project pin when using `--from`

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.1.x   | Yes       |
| 1.0.x   | Yes       |
| 0.2.x   | Yes (until 2026-06-01) |
| 0.1.x   | No        |

**Note**: Users on v0.2.2 or earlier should manually upgrade to a current 1.x release due to bugs in early `self-update` implementations. See [README.md](README.md) for upgrade instructions.
