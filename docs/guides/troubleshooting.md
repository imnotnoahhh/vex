# Troubleshooting Guide

This guide helps you diagnose and fix common issues with vex.

## Quick Diagnosis

Run `vex doctor` first:

```bash
vex doctor
```

This checks:
- vex installation and directory structure
- PATH configuration
- Shell hook setup
- Installed tools and symlinks
- Binary executability
- Network connectivity

Follow the suggestions provided by `vex doctor`.

## Common Issues

### Installation Issues

#### vex command not found

**Symptoms**: Shell can't find `vex` after installation.

**Diagnosis**:

```bash
# Check if vex exists
ls -la ~/.local/bin/vex

# Check if ~/.local/bin is in PATH
echo $PATH | grep -q "$HOME/.local/bin" && echo "✓ In PATH" || echo "✗ Not in PATH"
```

**Solutions**:

1. **Add to PATH**:
   ```bash
   # For zsh
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc

   # For bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

2. **Verify installation**:
   ```bash
   ~/.local/bin/vex --version
   ```

3. **Reinstall if needed**:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash
   ```

#### Permission denied

**Symptoms**: `vex: permission denied` when running.

**Diagnosis**:

```bash
ls -la ~/.local/bin/vex
# Should show: -rwxr-xr-x (executable)
```

**Solution**:

```bash
chmod +x ~/.local/bin/vex
```

#### Wrong architecture error

**Symptoms**: `Bad CPU type in executable` or similar error.

**Diagnosis**:

```bash
uname -m
# arm64 = Apple Silicon (M1/M2/M3)
# x86_64 = Intel

file ~/.local/bin/vex
# Should match your architecture
```

**Solution**:

Download the correct binary:
- Apple Silicon: `vex-aarch64-apple-darwin.tar.gz`
- Intel: `vex-x86_64-apple-darwin.tar.gz`

### Version Switching Issues

#### Versions don't auto-switch

**Symptoms**: `cd` into project but version doesn't change.

**Diagnosis**:

```bash
# Check if shell hook is installed
vex doctor

# Manually test auto-switch
vex use --auto

# Check if version file exists
cat .tool-versions
```

**Solutions**:

1. **Install shell hook**:
   ```bash
   # For zsh
   echo 'eval "$(vex env zsh)"' >> ~/.zshrc
   source ~/.zshrc
   ```

2. **Install required version**:
   ```bash
   vex install node@20.11.0
   ```

3. **Check version file syntax**:
   ```bash
   # Correct format
   node 20.11.0
   go 1.23.5

   # Incorrect (no @ symbol)
   node@20.11.0  # ✗ Wrong
   ```

#### which shows old path after switching

**Symptoms**: `which node` shows old path, but `node --version` is correct.

**Explanation**: Shell caches command locations for performance.

**Solution**:

```bash
# Clear shell command cache
hash -r

# Or restart shell
exec $SHELL
```

#### Symlink errors

**Symptoms**: `vex use` fails with symlink errors.

**Diagnosis**:

```bash
# Check symlinks
ls -la ~/.vex/bin/
ls -la ~/.vex/current/

# Check for broken symlinks
find ~/.vex -type l ! -exec test -e {} \; -print
```

**Solutions**:

1. **Remove broken symlinks**:
   ```bash
   find ~/.vex/bin -type l ! -exec test -e {} \; -delete
   find ~/.vex/current -type l ! -exec test -e {} \; -delete
   ```

2. **Reinstall version**:
   ```bash
   vex uninstall node@20.11.0
   vex install node@20.11.0
   ```

### Download and Installation Issues

#### Network timeout

**Symptoms**: Download fails with timeout error.

**Diagnosis**:

```bash
# Test network connectivity
ping -c 3 nodejs.org

# Check vex doctor
vex doctor
```

**Solutions**:

1. **Check internet connection**
2. **Retry** (vex automatically retries 3 times)
3. **Check firewall settings**
4. **Use VPN if needed**

#### Checksum verification failed

**Symptoms**: Installation fails with checksum mismatch.

**Explanation**: Downloaded file is corrupted or tampered with.

**Solutions**:

1. **Retry download** (file may have been corrupted during download):
   ```bash
   vex install node@20.11.0
   ```

2. **Clear cache and retry**:
   ```bash
   rm -rf ~/.vex/cache/*
   vex install node@20.11.0
   ```

3. **Check network stability** (unstable connection can corrupt downloads)

#### Disk space insufficient

**Symptoms**: Installation fails with disk space error.

**Diagnosis**:

```bash
# Check available disk space
df -h ~
```

**Solutions**:

1. **Free up space**:
   ```bash
   # Remove old versions
   vex list node
   vex uninstall node@18.0.0

   # Empty trash
   # Clear downloads folder
   ```

2. **Check disk usage**:
   ```bash
   du -sh ~/.vex/toolchains/*
   ```

#### Path traversal error

**Symptoms**: Installation fails with path validation error.

**Explanation**: Malicious or malformed archive detected.

**Solutions**:

1. **Report the issue** (this is a security feature)
2. **Verify download source** (ensure you're using official vex)
3. **Check for malware**

### Tool-Specific Issues

#### Node.js: npm not found

**Symptoms**: `node` works but `npm` doesn't.

**Diagnosis**:

```bash
ls -la ~/.vex/bin/ | grep npm
ls -la ~/.vex/current/node/bin/ | grep npm
```

**Solutions**:

1. **Reinstall Node.js**:
   ```bash
   vex uninstall node@20.11.0
   vex install node@20.11.0
   ```

2. **Check symlinks**:
   ```bash
   vex use node@20.11.0
   ```

#### Go: GOROOT issues

**Symptoms**: Go complains about GOROOT.

**Solution**:

Don't set `GOROOT` manually. vex manages this automatically.

```bash
# Remove GOROOT from shell config
# ~/.zshrc or ~/.bashrc
unset GOROOT
```

#### Java: JAVA_HOME not set

**Symptoms**: Tools complain about missing JAVA_HOME.

**Solution**:

Set `JAVA_HOME` to point to vex's Java installation:

```bash
# For zsh
echo 'export JAVA_HOME="$HOME/.vex/current/java/Contents/Home"' >> ~/.zshrc
source ~/.zshrc

# For bash
echo 'export JAVA_HOME="$HOME/.vex/current/java/Contents/Home"' >> ~/.bashrc
source ~/.bashrc
```

#### Rust: rustc not found

**Symptoms**: `rustc` command not found after installation.

**Diagnosis**:

```bash
ls -la ~/.vex/bin/ | grep rustc
ls -la ~/.vex/current/rust/
```

**Solutions**:

1. **Reinstall Rust**:
   ```bash
   vex uninstall rust@1.93.1
   vex install rust@stable
   ```

2. **Check PATH**:
   ```bash
   echo $PATH | grep vex
   ```

### Configuration Issues

#### Cache not working

**Symptoms**: vex always fetches fresh version lists.

**Diagnosis**:

```bash
# Check cache directory
ls -la ~/.vex/cache/remote_versions/

# Check config
cat ~/.vex/config.toml
```

**Solutions**:

1. **Check cache TTL**:
   ```toml
   # ~/.vex/config.toml
   cache_ttl_secs = 300  # 5 minutes
   ```

2. **Clear cache**:
   ```bash
   rm -rf ~/.vex/cache/remote_versions/*
   ```

#### Config file not found

**Symptoms**: vex complains about missing config.

**Solution**:

```bash
# Reinitialize vex
vex init
```

### Lock File Issues

#### Installation locked

**Symptoms**: Installation fails with "already in progress" error.

**Explanation**: Another vex process is installing the same version, or a stale lock exists.

**Diagnosis**:

```bash
ls -la ~/.vex/locks/
```

**Solutions**:

1. **Wait for other installation to finish**

2. **Remove stale lock** (if no other vex process is running):
   ```bash
   rm ~/.vex/locks/node-20.11.0.lock
   ```

3. **Check for running vex processes**:
   ```bash
   ps aux | grep vex
   ```

### Shell-Specific Issues

#### zsh: hook not working

**Diagnosis**:

```bash
# Check if hook is loaded
typeset -f __vex_use_if_found

# Check if chpwd hook is registered
echo $chpwd_functions
```

**Solutions**:

1. **Reload shell config**:
   ```bash
   source ~/.zshrc
   ```

2. **Check hook order** (load vex after Oh My Zsh):
   ```bash
   # ~/.zshrc
   source $ZSH/oh-my-zsh.sh
   eval "$(vex env zsh)"  # After Oh My Zsh
   ```

#### bash: PROMPT_COMMAND conflicts

**Diagnosis**:

```bash
echo $PROMPT_COMMAND
```

**Solution**:

Ensure vex hook is in PROMPT_COMMAND:

```bash
# Should contain __vex_check_dir_change
echo $PROMPT_COMMAND | grep vex
```

#### fish: function not found

**Diagnosis**:

```bash
functions __vex_use_if_found
```

**Solution**:

```bash
# Reload fish config
source ~/.config/fish/config.fish
```

#### nushell: hook not running

**Diagnosis**:

```bash
# Check if vex.nu exists
ls -la ~/.config/nushell/vex.nu

# Check if sourced in config
cat ~/.config/nushell/config.nu | grep vex
```

**Solution**:

```bash
# Regenerate hook
vex env nu | save -f ~/.config/nushell/vex.nu

# Ensure it's sourced
echo 'source ~/.config/nushell/vex.nu' >> ~/.config/nushell/config.nu
```

## Advanced Troubleshooting

### Enable debug output

For detailed debugging, run vex commands with verbose output:

```bash
# Set RUST_LOG environment variable
RUST_LOG=debug vex install node@20

# Or for specific modules
RUST_LOG=vex::installer=debug vex install node@20
```

### Check file permissions

```bash
# vex directory should be owned by you
ls -la ~/.vex/

# Fix permissions if needed
chmod -R u+rwX ~/.vex/
```

### Verify binary integrity

```bash
# Check if vex binary is corrupted
file ~/.local/bin/vex
# Should show: Mach-O 64-bit executable

# Reinstall if corrupted
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash
```

### Clean slate reinstall

If all else fails, completely remove and reinstall:

```bash
# 1. Remove vex completely
rm -rf ~/.vex
rm -f ~/.local/bin/vex

# 2. Remove shell hooks from config files
# Edit ~/.zshrc, ~/.bashrc, etc. and remove vex lines

# 3. Restart shell
exec $SHELL

# 4. Reinstall vex
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash

# 5. Initialize
vex init

# 6. Set up shell integration
echo 'eval "$(vex env zsh)"' >> ~/.zshrc
source ~/.zshrc
```

## Error Messages

### Common error messages and solutions

| Error | Cause | Solution |
|-------|-------|----------|
| `Tool not found: python` | Tool not supported | Check supported tools: node, go, java, rust |
| `Version not found: 99.0.0` | Invalid version | Use `vex list-remote <tool>` to see available versions |
| `Disk space insufficient` | Not enough disk space | Free up at least 500 MB |
| `Checksum verification failed` | Corrupted download | Retry or clear cache |
| `Path traversal detected` | Malicious archive | Report issue, verify source |
| `Installation already in progress` | Lock file exists | Wait or remove stale lock |
| `Home directory not found` | $HOME not set | Set HOME environment variable |

## Getting Help

If you can't resolve the issue:

1. **Run vex doctor**:
   ```bash
   vex doctor
   ```

2. **Check existing issues**:
   - Search [GitHub Issues](https://github.com/imnotnoahhh/vex/issues)

3. **File a new issue** with:
   - Output of `vex doctor`
   - Output of `vex --version`
   - Your macOS version: `sw_vers`
   - Your shell: `echo $SHELL`
   - Steps to reproduce
   - Error messages

4. **Include relevant logs**:
   ```bash
   RUST_LOG=debug vex <command> 2>&1 | tee vex-debug.log
   ```

## Prevention Tips

- **Run vex doctor regularly** to catch issues early
- **Keep vex updated** to get bug fixes
- **Don't manually edit ~/.vex/** (use vex commands)
- **Back up .tool-versions files** in version control
- **Use specific versions** in .tool-versions (not aliases)

## Next Steps

- [Getting Started Guide](getting-started.md) - Learn basic usage
- [Installation Guide](installation.md) - Detailed installation instructions
- [Shell Integration Guide](shell-integration.md) - Advanced shell setup
- [Main README](../../README.md) - Full feature list
