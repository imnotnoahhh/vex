# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Revised public-facing documentation for consistency and clarity: fully translated `CONTRIBUTING.md` to English and standardized wording in `README.md` and `SECURITY.md`.
- Updated public docs to use the real repository URL (`imnotnoahhh/vex`) instead of placeholder links.
- Clarified `vex list-remote` behavior in README (`interactive latest 20` by default, `--all` for full output).
- Added documentation notes for Go/Rust upstream remote-list limits and contributor-facing doc organization rules.
- Added GitHub Releases installation guidance to README, alongside the existing source-build installation path.
- Added a one-line release installer script (`scripts/install-release.sh`) that downloads the matching macOS artifact and updates shell PATH config.

## [0.1.0] - 2026-02-27

### Added

- Multi-language version management: Node.js, Go, Java (Eclipse Temurin), Rust
- Symlink-based version switching (no shim overhead)
- Fuzzy version matching (`node@20` resolves to latest 20.x, `node@lts` to latest LTS)
- Interactive version selection with `vex install <tool>`
- `.tool-versions` support for per-project version pinning
- `vex install` without arguments installs all tools from `.tool-versions`
- `vex local` / `vex global` commands to write `.tool-versions` files
- Shell hooks for zsh and bash (auto-switch on `cd`)
- SHA256 checksum verification for Node.js downloads
- Download progress bar with speed indicator
- `vex current` to show all active versions
- macOS support (Apple Silicon and Intel)
