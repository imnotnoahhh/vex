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

1. **Path Traversal Protection** (v0.1.6+)
   - Validates all archive entry paths before extraction
   - Rejects paths containing `..` (parent directory references)
   - Rejects absolute paths
   - Prevents zip-slip style attacks

2. **HTTP Timeout Configuration** (v0.1.6+)
   - Connection timeout: 30 seconds
   - Total timeout: 5 minutes
   - Automatic retry: 3 attempts with 2-second intervals
   - 4xx client errors (e.g., 404) are not retried
   - Prevents indefinite hangs and resource exhaustion

3. **Disk Space Check** (v0.1.6+)
   - Validates at least 500 MB free space before installation
   - Prevents partial installations on full disks
   - Mitigates disk space exhaustion DoS attacks

4. **Checksum Verification**
   - SHA256 verification for Node.js downloads
   - Go, Java, and Rust follow upstream checksum metadata availability
   - Detects corrupted or tampered downloads

5. **Installation Locking**
   - File-based locking prevents concurrent installation corruption
   - Automatic cleanup of stale locks

6. **Error Handling**
   - Actionable error messages with troubleshooting steps
   - No sensitive information leaked in error messages

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |
