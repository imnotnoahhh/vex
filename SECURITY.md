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
- Symlink traversal or path injection in `~/.vex/`
- Download integrity bypass (SHA256 checksum verification)
- Arbitrary code execution via crafted version strings or `.tool-versions` files
- Supply chain risks in downloaded toolchain binaries

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |
