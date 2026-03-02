# Shell Integration Guide

This guide explains how to set up and customize vex shell integration for automatic version switching.

## Overview

Shell integration allows vex to automatically switch tool versions when you `cd` into a directory with a `.tool-versions` file. This happens transparently without any manual intervention.

## How It Works

When you `cd` into a directory, vex:

1. Traverses up the directory tree looking for version files
2. Finds `.tool-versions` (or `.node-version`, `.go-version`, etc.)
3. Checks if the required versions are installed
4. Switches to those versions automatically (if installed)
5. Silently skips if versions are not installed

## Supported Shells

vex supports four shells:

- **zsh** (default on macOS Catalina and later)
- **bash** (older macOS versions, Linux)
- **fish** (Fish shell)
- **nushell** (Nushell)

## Setup Instructions

### zsh

**Add to `~/.zshrc`:**

```bash
eval "$(vex env zsh)"
```

**Apply changes:**

```bash
source ~/.zshrc
```

**How it works:**

vex uses zsh's `chpwd` hook, which runs every time you change directories:

```bash
autoload -U add-zsh-hook
add-zsh-hook chpwd __vex_use_if_found
```

### bash

**Add to `~/.bashrc` or `~/.bash_profile`:**

```bash
eval "$(vex env bash)"
```

**Apply changes:**

```bash
source ~/.bashrc
```

**How it works:**

vex uses bash's `PROMPT_COMMAND`, which runs before each prompt:

```bash
PROMPT_COMMAND="__vex_check_dir_change; $PROMPT_COMMAND"
```

It tracks the previous directory and only runs when the directory changes.

### fish

**Add to `~/.config/fish/config.fish`:**

```bash
vex env fish | source
```

**Apply changes:**

```bash
source ~/.config/fish/config.fish
```

**How it works:**

vex uses fish's event system to monitor the `PWD` variable:

```fish
function __vex_on_pwd --on-variable PWD
    __vex_use_if_found
end
```

### nushell

**Create vex hook file:**

```bash
vex env nu | save -f ~/.config/nushell/vex.nu
```

**Add to `~/.config/nushell/config.nu`:**

```bash
source ~/.config/nushell/vex.nu
```

**Apply changes:**

Restart nushell or run:

```bash
source ~/.config/nushell/config.nu
```

**How it works:**

vex uses nushell's `pre_prompt` hooks:

```nu
$env.config.hooks.pre_prompt = ($env.config.hooks.pre_prompt | append {||
    __vex_use_if_found
})
```

## Verifying Shell Integration

### Check if hook is installed

**For zsh:**

```bash
typeset -f __vex_use_if_found
# Should output the function definition
```

**For bash:**

```bash
declare -f __vex_use_if_found
# Should output the function definition
```

**For fish:**

```bash
functions __vex_use_if_found
# Should output the function definition
```

**For nushell:**

```bash
which __vex_use_if_found
# Should show the function
```

### Test auto-switching

```bash
# Create test directory
mkdir -p /tmp/vex-test
cd /tmp/vex-test

# Create .tool-versions
echo "node 20.11.0" > .tool-versions

# Install the version
vex install node@20.11.0

# cd into directory (should auto-switch)
cd /tmp/vex-test

# Verify
vex current
# node 20.11.0
```

## Advanced Configuration

### Disable Auto-Switching

If you want to disable auto-switching temporarily:

**For zsh:**

```bash
# Remove the hook
add-zsh-hook -d chpwd __vex_use_if_found
```

**For bash:**

```bash
# Remove from PROMPT_COMMAND
PROMPT_COMMAND="${PROMPT_COMMAND//__vex_check_dir_change;/}"
```

**For fish:**

```bash
# Remove the function
functions -e __vex_on_pwd
```

**For nushell:**

Remove the hook from `~/.config/nushell/config.nu` and restart.

### Custom Hook Behavior

You can customize the hook by editing the generated script.

**View the generated hook:**

```bash
vex env zsh  # or bash, fish, nu
```

**Customize the hook:**

1. Generate the hook: `vex env zsh > /tmp/vex-hook.sh`
2. Edit `/tmp/vex-hook.sh` to customize behavior
3. Source it in your shell config: `source /tmp/vex-hook.sh`

**Example customization** (show notification on switch):

```bash
__vex_use_if_found() {
    local dir="$PWD"
    while [ "$dir" != "" ]; do
        if [ -f "$dir/.tool-versions" ]; then
            echo "🔄 Switching versions..."
            vex use --auto 2>/dev/null
            return
        fi
        dir="${dir%/*}"
    done
}
```

## Version File Priority

vex checks for version files in this order:

1. `.tool-versions` (highest priority)
2. `.node-version` / `.nvmrc`
3. `.go-version`
4. `.java-version`
5. `.rust-toolchain`

If `.tool-versions` exists, language-specific files are ignored.

## Performance Considerations

### Hook Performance

The vex hook is designed to be fast:

- **Directory traversal**: Stops at first version file found
- **File checks**: Uses fast filesystem operations
- **Silent execution**: Redirects stderr to `/dev/null`
- **Lazy evaluation**: Only runs when directory changes

### Benchmarks

Typical hook execution time:

- **With version file**: ~10-20ms
- **Without version file**: ~5-10ms (traversal only)

This is fast enough to be imperceptible in normal shell usage.

### Optimization Tips

1. **Keep .tool-versions in project root**: Reduces traversal time
2. **Don't nest projects deeply**: Shorter paths = faster traversal
3. **Use SSD**: Faster filesystem operations

## Troubleshooting

### Auto-switching not working

**Problem**: Versions don't switch when you `cd`.

**Diagnosis**:

```bash
# Check if hook is installed
vex doctor

# Manually test the hook
__vex_use_if_found
```

**Solutions**:

1. **Hook not installed**: Re-run setup instructions
2. **Shell config not sourced**: Restart shell or source config
3. **Version not installed**: Install with `vex install`

### Hook runs but doesn't switch

**Problem**: Hook executes but version doesn't change.

**Diagnosis**:

```bash
# Run manually with output
vex use --auto
```

**Solutions**:

1. **Version not installed**: Install the required version
2. **Invalid version file**: Check `.tool-versions` syntax
3. **Permission issues**: Check file permissions

### Slow shell startup

**Problem**: Shell takes long to start after adding vex hook.

**Diagnosis**:

```bash
# Time shell startup
time zsh -i -c exit
```

**Solutions**:

1. **Hook is not the issue**: vex hook adds <1ms to startup
2. **Other plugins**: Check other shell plugins
3. **Profile shell startup**: Use `zprof` (zsh) or similar tools

### Conflicts with other version managers

**Problem**: vex conflicts with nvm, asdf, or other version managers.

**Solutions**:

1. **Remove other version managers**: Uninstall nvm, asdf, etc.
2. **Adjust PATH order**: Ensure `~/.vex/bin` comes first
3. **Use only one manager**: Don't mix version managers

### Shell cache issues

**Problem**: `which node` shows old path after switching.

**Explanation**: Shell caches command locations for performance.

**Solution**:

```bash
# Clear shell command cache
hash -r

# Or restart shell
exec $SHELL
```

vex automatically suggests this when needed.

## Multiple Shells

If you use multiple shells, set up vex for each:

```bash
# zsh
echo 'eval "$(vex env zsh)"' >> ~/.zshrc

# bash
echo 'eval "$(vex env bash)"' >> ~/.bashrc

# fish
echo 'vex env fish | source' >> ~/.config/fish/config.fish

# nushell
vex env nu | save -f ~/.config/nushell/vex.nu
echo 'source ~/.config/nushell/vex.nu' >> ~/.config/nushell/config.nu
```

## Shell-Specific Tips

### zsh

- **Oh My Zsh**: vex works with Oh My Zsh, add hook after `source $ZSH/oh-my-zsh.sh`
- **Powerlevel10k**: Compatible, no special configuration needed
- **Plugins**: Load vex after other plugins that modify PATH

### bash

- **macOS**: Use `~/.bash_profile` instead of `~/.bashrc`
- **Linux**: Use `~/.bashrc`
- **Login vs non-login**: Add to both if unsure

### fish

- **Plugins**: Compatible with fisher, oh-my-fish, etc.
- **Universal variables**: vex doesn't use universal variables
- **Abbreviations**: Create abbreviations for common vex commands

### nushell

- **Config location**: Ensure `~/.config/nushell/config.nu` exists
- **Hooks**: vex appends to existing `pre_prompt` hooks
- **Environment**: vex modifies `$env.PATH` correctly

## Next Steps

- [Getting Started Guide](getting-started.md) - Learn basic usage
- [Troubleshooting Guide](troubleshooting.md) - Common issues and solutions
- [Main README](../../README.md) - Full feature list

## Getting Help

If shell integration isn't working:

1. Run `vex doctor` to diagnose
2. Check the [Troubleshooting Guide](troubleshooting.md)
3. File an issue with:
   - Your shell and version
   - Output of `vex doctor`
   - Your shell config file
