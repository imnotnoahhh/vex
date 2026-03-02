# vex Documentation

Welcome to the vex documentation! This directory contains guides and resources to help you get the most out of vex.

## Quick Links

- [Getting Started Guide](guides/getting-started.md) - New to vex? Start here!
- [Installation Guide](guides/installation.md) - Detailed installation instructions
- [Shell Integration Guide](guides/shell-integration.md) - Set up auto-switching for your shell
- [Troubleshooting Guide](guides/troubleshooting.md) - Common issues and solutions

## Main Documentation

- [README.md](../README.md) - Project overview and quick start
- [CONTRIBUTING.md](../CONTRIBUTING.md) - How to contribute to vex
- [ARCHITECTURE.md](../ARCHITECTURE.md) - System architecture and design decisions
- [TESTING.md](../TESTING.md) - Testing guidelines and best practices
- [RELEASING.md](../RELEASING.md) - Release process and versioning
- [SECURITY.md](../SECURITY.md) - Security policy and reporting vulnerabilities
- [CHANGELOG.md](../CHANGELOG.md) - Version history and release notes

## API Documentation

Generate and view the Rust API documentation:

```bash
make docs
```

Or manually:

```bash
RUSTDOCFLAGS="--html-in-header docs/header.html" cargo doc --no-deps
cp docs/custom.css target/doc/
open target/doc/vex/index.html
```

The API documentation includes:
- Module-level documentation explaining purpose and architecture
- Function documentation with parameters, returns, and errors
- Type documentation for structs, enums, and traits
- Code examples and usage notes

## Documentation Files

### User Guides (`guides/`)

- **getting-started.md** - Quick start guide for new users
- **installation.md** - Detailed installation instructions for all methods
- **shell-integration.md** - Shell hook setup for zsh, bash, fish, and nushell
- **troubleshooting.md** - Common problems and how to fix them

### Developer Documentation

- **custom.css** - Custom styling for generated Rustdoc
- **header.html** - HTML header for Rustdoc customization
- **demo/** - Demo files and recordings

### Archived Documentation (`archive/`)

Historical development documents (not tracked in git):
- Design documents
- Implementation plans
- Development notes
- Translation summaries

## Contributing to Documentation

Found a typo or want to improve the docs? Contributions are welcome!

1. **User guides**: Edit files in `docs/guides/`
2. **API docs**: Edit doc comments in source files (`src/**/*.rs`)
3. **Main docs**: Edit files in project root (README.md, CONTRIBUTING.md, etc.)

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## Building Documentation Locally

### User Guides

User guides are written in Markdown and can be viewed in any Markdown viewer or on GitHub.

### API Documentation

```bash
# Generate documentation
cargo doc --no-deps

# With custom styling
RUSTDOCFLAGS="--html-in-header docs/header.html" cargo doc --no-deps
cp docs/custom.css target/doc/

# Open in browser
open target/doc/vex/index.html
```

### Using Makefile

```bash
# Generate and serve documentation
make docs

# Stop documentation server
make docs-stop
```

## Documentation Standards

### Writing Style

- **Clear and concise**: Use simple language, avoid jargon
- **Action-oriented**: Start with verbs (Install, Configure, Run)
- **Examples**: Include code examples and command outputs
- **Troubleshooting**: Anticipate common issues and provide solutions

### Formatting

- **Headings**: Use ATX-style headings (`#`, `##`, `###`)
- **Code blocks**: Always specify language (```bash, ```rust, ```toml)
- **Links**: Use relative links for internal documentation
- **Lists**: Use `-` for unordered lists, `1.` for ordered lists

### Code Examples

```bash
# Good: Include comments and expected output
vex install node@20
# Installing Node.js 20.11.0...
# ✓ Downloaded and verified
# ✓ Installed to ~/.vex/toolchains/node/20.11.0
# ✓ Switched to node 20.11.0

# Bad: No context or explanation
vex install node@20
```

## Getting Help

- **Documentation**: Check the guides in this directory
- **Issues**: Search [GitHub Issues](https://github.com/imnotnoahhh/vex/issues)
- **Discussions**: Ask questions in [GitHub Discussions](https://github.com/imnotnoahhh/vex/discussions)
- **Bug Reports**: File an issue with reproduction steps

## License

Documentation is licensed under [MIT](../LICENSE), same as the project.
