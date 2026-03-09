# Homebrew Integration

This directory contains the Homebrew formula template and automation for vex.

## Files

- `vex.rb.template`: Homebrew formula template with placeholders for version and SHA256 checksums
- `README.md`: This file

## Automatic Updates

The `.github/workflows/release.yml` workflow automatically updates the Homebrew formula when a new release is published:

1. Downloads SHA256 checksums for both ARM and Intel builds
2. Replaces placeholders in the template:
   - `VERSION_PLACEHOLDER` → actual version (e.g., `0.2.3`)
   - `AARCH64_SHA256` → SHA256 for ARM build
   - `X86_64_SHA256` → SHA256 for Intel build
3. Commits and pushes to `qinfuyao/homebrew-vex` repository

## Setup Requirements

### 1. Create Homebrew Tap Repository

Create a new GitHub repository named `homebrew-vex` with the following structure:

```
homebrew-vex/
└── Formula/
    └── vex.rb
```

### 2. Configure GitHub Token

Add a GitHub Personal Access Token to repository secrets:

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Generate new token with `repo` scope (full control of private repositories)
3. Add the token to vex repository secrets as `HOMEBREW_TAP_TOKEN`:
   - Go to vex repository → Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `HOMEBREW_TAP_TOKEN`
   - Value: your generated token

### 3. Verify Workflow

After setting up the token, the next release will automatically:
- Update the formula in `homebrew-vex` repository
- Users can install via: `brew install qinfuyao/vex/vex`

## Manual Update

If automatic update fails or `HOMEBREW_TAP_TOKEN` is not configured:

1. Download the generated formula from workflow artifacts
2. Manually commit to `homebrew-vex/Formula/vex.rb`

## Formula Features

- **Multi-architecture support**: Automatically selects ARM or Intel build based on CPU
- **Installation**: Installs `vex` binary to Homebrew's bin directory
- **Post-install message**: Shows shell configuration instructions
- **Tests**: Verifies installation with `--version`, `--help`, and `list` commands

## Testing Locally

To test the formula locally before release:

```bash
# Replace placeholders manually
sed -e "s/VERSION_PLACEHOLDER/0.2.3/g" \
    -e "s/AARCH64_SHA256/actual_sha256_here/g" \
    -e "s/X86_64_SHA256/actual_sha256_here/g" \
    homebrew/vex.rb.template > /tmp/vex.rb

# Install from local formula
brew install --build-from-source /tmp/vex.rb

# Test
vex --version
```

## Troubleshooting

### Formula update fails

Check workflow logs for:
- Network issues downloading SHA256 files
- Invalid token permissions
- Repository access issues

### SHA256 mismatch

Ensure the release workflow completed successfully and SHA256 files are uploaded to GitHub releases.

### Token expired

Regenerate the Personal Access Token and update `HOMEBREW_TAP_TOKEN` secret.
