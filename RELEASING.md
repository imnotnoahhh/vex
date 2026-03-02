# Release Process

This document describes the release process for vex.

## Table of Contents

- [Versioning](#versioning)
- [Release Checklist](#release-checklist)
- [Creating a Release](#creating-a-release)
- [CI/CD Pipeline](#cicd-pipeline)
- [Post-Release](#post-release)

## Versioning

vex follows [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: Incompatible API changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Version Examples

- `0.1.0` → `0.1.1`: Bug fix
- `0.1.1` → `0.2.0`: New feature (e.g., Python support)
- `0.2.0` → `1.0.0`: Stable API, breaking changes

### Pre-1.0 Versioning

During the 0.x phase:
- Breaking changes are allowed in MINOR versions
- PATCH versions are for bug fixes only
- Document breaking changes clearly in CHANGELOG.md

## Release Checklist

### 1. Pre-Release Checks

- [ ] All tests pass locally
  ```bash
  cargo test --all-features
  cargo test --all-features -- --ignored  # Network tests
  ```

- [ ] Code is formatted
  ```bash
  cargo fmt --all -- --check
  ```

- [ ] No clippy warnings
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```

- [ ] No security vulnerabilities
  ```bash
  cargo audit
  ```

- [ ] Documentation builds without warnings
  ```bash
  cargo doc --no-deps
  ```

- [ ] Benchmarks run successfully (optional)
  ```bash
  cargo bench
  ```

### 2. Update Version Numbers

Update version in the following files:

- [ ] `Cargo.toml`
  ```toml
  [package]
  version = "0.1.7"
  ```

- [ ] `Cargo.lock` (run `cargo build` to update)
  ```bash
  cargo build
  ```

### 3. Update CHANGELOG.md

- [ ] Move changes from `[Unreleased]` to new version section
- [ ] Add release date
- [ ] Ensure all changes are documented
- [ ] Follow [Keep a Changelog](https://keepachangelog.com/) format

**Example**:
```markdown
## [Unreleased]

## [0.1.7] - 2026-03-15

### Added
- Python support via python-build-standalone

### Fixed
- Node.js checksum verification on slow networks

### Changed
- Improved error messages for disk space issues
```

### 4. Update Documentation

- [ ] Update README.md if needed
  - New features
  - New commands
  - Installation instructions

- [ ] Update CLAUDE.md if needed
  - New implementation details
  - Updated architecture

- [ ] Update ARCHITECTURE.md if needed
  - New modules
  - Changed data flows

### 5. Commit Changes

Create a release preparation commit:

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md README.md
git commit -m "chore: prepare v0.1.7 release"
```

### 6. Create Pull Request

- [ ] Open PR to `main` branch
- [ ] Title: `chore: prepare v0.1.7 release`
- [ ] Description: Link to CHANGELOG section
- [ ] Wait for CI to pass
- [ ] Get approval from maintainer
- [ ] Merge PR

## Creating a Release

### 1. Tag the Release

After merging the release PR:

```bash
# Pull latest main
git checkout main
git pull origin main

# Create annotated tag
git tag -a v0.1.7 -m "Release v0.1.7"

# Push tag to GitHub
git push origin v0.1.7
```

### 2. GitHub Release

GitHub Actions will automatically:
1. Build binaries for macOS (arm64 and x86_64)
2. Create a GitHub Release
3. Upload binaries as release assets

**Manual steps** (if needed):

1. Go to https://github.com/imnotnoahhh/vex/releases
2. Click "Draft a new release"
3. Choose tag: `v0.1.7`
4. Release title: `v0.1.7`
5. Description: Copy from CHANGELOG.md
6. Attach binaries (if not automated):
   - `vex-aarch64-apple-darwin.tar.gz`
   - `vex-x86_64-apple-darwin.tar.gz`
7. Click "Publish release"

### 3. Verify Release

- [ ] Check GitHub Release page
- [ ] Download and test binaries
  ```bash
  # Download
  curl -LO https://github.com/imnotnoahhh/vex/releases/download/v0.1.7/vex-aarch64-apple-darwin.tar.gz

  # Extract
  tar -xzf vex-aarch64-apple-darwin.tar.gz

  # Test
  ./vex-aarch64-apple-darwin/vex --version
  ```

- [ ] Test installation script
  ```bash
  curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash -s -- --version v0.1.7
  ```

## CI/CD Pipeline

### GitHub Actions Workflows

#### 1. CI Workflow (`.github/workflows/ci.yml`)

Runs on every push and PR:

```yaml
jobs:
  fmt:      # Code formatting check
  clippy:   # Linting
  test:     # Unit and integration tests
  audit:    # Security audit
```

#### 2. Release Workflow (`.github/workflows/release.yml`)

Runs on tag push (`v*`):

```yaml
jobs:
  build:
    strategy:
      matrix:
        target:
          - aarch64-apple-darwin  # Apple Silicon
          - x86_64-apple-darwin   # Intel
    steps:
      - Build release binary
      - Create tarball
      - Upload to GitHub Release
```

### Release Artifacts

Each release includes:

- `vex-aarch64-apple-darwin.tar.gz` (Apple Silicon)
- `vex-x86_64-apple-darwin.tar.gz` (Intel)
- Source code (zip)
- Source code (tar.gz)

### Artifact Structure

```
vex-aarch64-apple-darwin/
├── vex           # Binary
├── README.md     # Installation instructions
└── LICENSE       # MIT license
```

## Post-Release

### 1. Announce Release

- [ ] Update project README badges (if needed)
- [ ] Post on social media (optional)
- [ ] Notify users in discussions (optional)

### 2. Monitor Issues

- [ ] Watch for bug reports related to new release
- [ ] Respond to installation issues
- [ ] Prepare hotfix if critical bugs found

### 3. Update Documentation

- [ ] Ensure docs.rs has latest documentation
- [ ] Update any external documentation links

### 4. Plan Next Release

- [ ] Create milestone for next version
- [ ] Label issues for next release
- [ ] Update roadmap if needed

## Hotfix Process

For critical bugs in production:

### 1. Create Hotfix Branch

```bash
git checkout -b hotfix/v0.1.7.1 v0.1.7
```

### 2. Fix Bug

```bash
# Make fix
git add .
git commit -m "fix: critical bug description"
```

### 3. Update Version

- Update `Cargo.toml` to `0.1.7.1` (PATCH bump)
- Update `CHANGELOG.md`

### 4. Release

```bash
# Commit version bump
git commit -am "chore: prepare v0.1.7.1 hotfix"

# Merge to main
git checkout main
git merge hotfix/v0.1.7.1

# Tag and push
git tag -a v0.1.7.1 -m "Hotfix v0.1.7.1"
git push origin main v0.1.7.1
```

## Rollback Process

If a release has critical issues:

### 1. Identify Problem

- Document the issue
- Assess severity
- Decide: hotfix or rollback?

### 2. Rollback Release (if needed)

```bash
# Delete tag locally
git tag -d v0.1.7

# Delete tag on GitHub
git push origin :refs/tags/v0.1.7

# Delete GitHub Release (manual)
# Go to Releases page and delete
```

### 3. Communicate

- Update GitHub Release with warning
- Post issue explaining the problem
- Notify users who may have installed

### 4. Fix and Re-Release

- Fix the issue
- Bump version (e.g., `0.1.7` → `0.1.8`)
- Follow normal release process

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.6 | 2026-03-02 | Fish/Nushell support, vex doctor, security improvements |
| 0.1.5 | 2026-03-01 | Colorful output |
| 0.1.4 | 2026-03-01 | Rust complete toolchain, uninstall cleanup |
| 0.1.3 | 2026-03-01 | Java and Rust support |
| 0.1.2 | 2026-02-28 | Go support, .tool-versions |
| 0.1.1 | 2026-02-28 | Shell hooks, auto-switch |
| 0.1.0 | 2026-02-27 | Initial release, Node.js support |

## References

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [cargo-release](https://github.com/crate-ci/cargo-release) (optional automation tool)
