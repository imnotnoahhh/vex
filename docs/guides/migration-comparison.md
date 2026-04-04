# Migration and Comparison Guide

This guide helps you move to `vex` from `nvm`, `asdf`, or `pyenv` without guessing about behavior that has not been verified.

It focuses on:

- current `vex` behavior in this repository
- official upstream documentation for `nvm`, `asdf`, `pyenv`, and `pyenv-virtualenv`
- migration steps that are safe to validate locally on macOS

If your current setup depends on custom plugins, shell wrappers, or private toolchains, treat this guide as the baseline and test those extras separately.

## At a Glance

| Manager | Scope | Common version file | Activation model | Main migration concern |
|---------|-------|---------------------|------------------|------------------------|
| `nvm` | Node.js only | `.nvmrc` | shell initialization plus `nvm use` | remove old shell init after verifying `vex` owns `PATH` |
| `asdf` | Multi-language via plugins | `.tool-versions` | shims plus plugin runtime | normalize `.tool-versions` for `vex` semantics and supported tools |
| `pyenv` | Python only | `.python-version` | shims plus `pyenv init` | replace shim-based Python selection and virtualenv plugin workflows |
| `vex` | Built-in support for Node.js, Go, Java, Rust, Python | `.tool-versions` plus selected language-specific files | direct symlink switching, `vex exec`, `vex run`, shell hook | macOS only, built-in tools only |

## Verified `vex` Behaviors That Matter During Migration

Before migrating, these are the `vex` rules worth anchoring on:

- `vex` supports the built-in tool names `node`, `go`, `java`, `rust`, and `python`.
- `vex` reads these project files: `.tool-versions`, `.node-version`, `.nvmrc`, `.go-version`, `.java-version`, `.rust-toolchain`, and `.python-version`.
- File priority is: `.tool-versions` first, then language-specific files.
- Project lookup walks up parent directories, so nested directories can inherit a root `.tool-versions` file.
- A child `.tool-versions` file overrides matching tools from a parent directory while leaving unrelated parent entries in place.
- Global defaults live in `~/.vex/tool-versions`, not `~/.tool-versions`.
- Legacy global files and supported language home/cache directories can be audited and migrated explicitly with `vex repair migrate-home`.
- `vex exec` and `vex run` activate the resolved environment for a single process without changing global symlinks.
- With the shell hook installed, `vex` also auto-activates a project-local `.venv` when you `cd` into that project.

## General Migration Checklist

Use this checklist no matter which manager you are migrating from:

1. Install `vex` and initialize shell integration.
2. Decide which manager will own `PATH` during the migration window. Do not leave both active long term.
3. Normalize your project version files into a format `vex` actually understands.
4. Install the required versions with `vex`.
5. Verify with `vex current`, `<tool> --version`, and `which <tool>`.
6. Clear your shell command cache with `hash -r` if `which` still shows the old path.
7. Only after verification, remove the old manager's shell initialization.

Recommended verification commands:

```bash
vex --version
vex doctor
vex current
which node
which python
echo "$PATH"
```

## Mental Model Shifts

The biggest change when moving to `vex` is not syntax. It is ownership.

- `vex` is not plugin-driven today. It has first-party support for a fixed set of tools.
- `vex` keeps version selection in version files and keeps tasks, env vars, and network settings in `.vex.toml`.
- `vex` prefers direct symlink switching over shim dispatch.
- `vex use` updates the active symlinks under `~/.vex` for the current user, so it is broader than a shell-only override such as `pyenv shell`.
- `vex exec` and `vex run` are the safest replacements for many one-off shell overrides and ad-hoc subshell workflows.

If your existing setup mixes version selection, shell aliases, task runners, and virtualenv activation in one place, split those responsibilities during migration instead of copying them verbatim.

## Coming from `nvm`

According to the official `nvm` documentation, `nvm` installs under `~/.nvm`, is initialized from your shell profile, and uses `.nvmrc` as its project version file. Its documented auto-use behavior is a shell recipe, not a universal built-in for every shell session.

### What maps cleanly to `vex`

- `.nvmrc` can stay in place temporarily because `vex` reads it for Node.js resolution.
- `nvm use` maps conceptually to either:
  - `vex use node@<version>` for explicit switching
  - `vex` shell auto-switching when a version file is present
- `nvm exec` style one-off commands map better to `vex exec -- <command>`.
- `nvm alias default` maps conceptually to `vex global node@<version>`.

### What changes

- `vex` is multi-language, so `.tool-versions` becomes the better long-term file if the repo needs more than Node.js.
- `vex` does not store its state under `~/.nvm`.
- `vex` shell ownership should replace `nvm` shell ownership once migration is complete.

### Suggested migration flow

1. Install `vex` and run:

   ```bash
   vex init --shell auto
   vex doctor
   ```

2. If the repository is Node-only and already has a correct `.nvmrc`, you can keep that file for the first validation pass.

3. If the repository will use more than Node.js, create or update `.tool-versions`:

   ```text
   node 20.11.0
   ```

4. Install the required Node.js version:

   ```bash
   vex install node@20.11.0
   ```

5. Verify:

   ```bash
   vex current
   node --version
   which node
   ```

6. Remove `nvm` initialization lines from your shell config only after verification.

### Notes and gotchas

- Do not keep both `nvm` init lines and `vex` shell hook active indefinitely.
- If a repo already has both `.tool-versions` and `.nvmrc`, `vex` will prefer `.tool-versions`.
- If your old shell still resolves `node` from an `nvm` path after switching, run `hash -r` or restart the shell.

## Coming from `asdf`

According to the official `asdf` docs, `asdf` is a plugin-based version manager that uses shims and a `.tool-versions` file. Its version file format can express multiple fallback versions, `system`, `ref:`, and `path:` values, and global defaults live in `~/.tool-versions`.

That means `asdf` is the closest conceptual match to `vex`, but it is also the migration with the most hidden format differences.

### What maps cleanly to `vex`

- The idea of a checked-in `.tool-versions` file carries over well.
- Root-level shared defaults plus child-directory overrides also carry over well.
- The idea of a global default version file also carries over, but `vex` stores it at `~/.vex/tool-versions`.

### What changes

- `vex` does not use plugin names. It only recognizes `node`, `go`, `java`, `rust`, and `python`.
- `vex` does not implement the broader `asdf` version expression model. Rewrite any line that relies on:
  - multiple fallback versions
  - `system`
  - `ref:...`
  - `path:...`
  - unsupported tool names
- `vex` separates project tasks and env vars into `.vex.toml`; they should not be encoded into version-file conventions.
- `vex` does not require installing plugins before installing tool versions.

### Rewrite your `.tool-versions` file deliberately

Good `vex` input:

```text
node 20.11.0
go 1.24.0
python 3.12.8
```

Examples that should be rewritten before relying on `vex`:

```text
nodejs 20.11.0
python 3.12.8 system
node ref:main
terraform 1.7.5
```

The examples above are common in plugin-based `asdf` setups, but they are not valid `vex` migration targets as-is.

### Suggested migration flow

1. Audit every entry in your current `.tool-versions` file.

2. Rewrite it so that:
   - each line uses a `vex` tool name
   - each tool has one version or supported alias
   - unsupported tools are removed or managed outside `vex`

3. Preview legacy home-state cleanup and migrate the safe paths into `~/.vex` explicitly:

   ```bash
   vex repair migrate-home
   ```

4. Install and verify:

   ```bash
   vex install
   vex current
   ```

5. If you need project tasks or repo-local env vars, add them to `.vex.toml` instead of stretching `.tool-versions`.

6. Remove `asdf` initialization from your shell only after `which <tool>` resolves into `~/.vex/bin`.

### Notes and gotchas

- If your current `asdf` setup depends on plugins for tools outside `vex`'s built-in set, migrate only the supported tools first.
- If your repo uses `asdf` plugin-specific conventions, keep those repos on `asdf` until you have a clear replacement plan.
- If you benchmark `asdf` against `vex`, make sure you compare equivalent workflows and note that one is shim-based and one is symlink-based.

## Coming from `pyenv`

According to the official `pyenv` docs, `pyenv` uses shims, resolves versions from `PYENV_VERSION`, `.python-version`, parent directories, and a global version file, and supports `local`, `global`, and `shell` version selection. The official `pyenv-virtualenv` plugin adds virtualenv and auto-activation workflows on top of that.

### What maps cleanly to `vex`

- `.python-version` can stay in place temporarily because `vex` reads it.
- `pyenv local` maps conceptually to a project version file.
- `pyenv global` maps conceptually to `vex global python@<version>`.
- `pyenv-virtualenv` style "activate a project environment when I enter the directory" maps conceptually to `vex` shell hook plus a local `.venv`.

### What changes

- `vex` stores global defaults in `~/.vex/tool-versions`, not in `pyenv`'s global file location.
- `vex` does not expose a direct equivalent of `pyenv shell` or `PYENV_VERSION`.
- For one-off commands, `vex exec` is the cleaner replacement.
- For Python environments, `vex` uses project-local `.venv` plus:
  - `vex python init`
  - `vex python freeze`
  - `vex python sync`
- If your `.python-version` file contains multiple entries or other `pyenv`-specific patterns, rewrite it to a single version string before treating it as a `vex` source of truth.

### Suggested migration flow

1. Install the Python you want `vex` to manage:

   ```bash
   vex install python@3.12
   vex global python@3.12
   ```

2. For Python-only repos, you can keep `.python-version` during the initial migration.

3. For mixed-language repos, convert to `.tool-versions`:

   ```text
   python 3.12.8
   node 20.11.0
   ```

4. Replace virtualenv plugin workflows with `vex` commands:

   ```bash
   vex python init
   vex python freeze
   vex python sync
   ```

5. Verify:

   ```bash
   vex current
   python --version
   which python
   ```

6. Remove `pyenv init` and `pyenv virtualenv-init` lines after the environment behaves correctly under `vex`.

### Notes and gotchas

- If you depended on shell-scoped version overrides, replace them with explicit project files or `vex exec`.
- `vex` will auto-activate a `.venv` in the current project when the shell hook is installed, but it does not try to mirror every `pyenv-virtualenv` feature.
- Keep `.venv` uncommitted and commit the files that describe it: `.tool-versions` and `requirements.lock`.

## Version File Compatibility Summary

Use this table when deciding what to keep and what to rewrite.

| File | `vex` reads it | Best long-term use with `vex` |
|------|----------------|-------------------------------|
| `.tool-versions` | Yes | preferred for checked-in multi-tool projects |
| `.nvmrc` | Yes, for Node.js | acceptable for temporary migration or Node-only repos |
| `.node-version` | Yes, for Node.js | acceptable when already present |
| `.python-version` | Yes, for Python | acceptable for temporary migration or Python-only repos |
| `~/.tool-versions` | auto-migrated to `~/.vex/tool-versions` when possible | move to `~/.vex/tool-versions` |

The recommended end state for most active `vex` repositories is:

- `.tool-versions` for project version pins
- `.tool-versions.lock` for reproducible installs
- `.vex.toml` for project tasks, env vars, and repo-local configuration

## Final Verification Before Removing the Old Manager

Do not uninstall the old manager until these checks pass:

```bash
vex doctor
vex current
node --version
python --version
which node
which python
```

You should see active binaries resolving from `~/.vex/bin` or from a `vex`-managed toolchain path, not from `~/.nvm`, `~/.asdf`, or `~/.pyenv`.

## Official Upstream References

These are the upstream references used when writing this guide:

- [`nvm` README](https://github.com/nvm-sh/nvm)
- [`asdf` Getting Started](https://asdf-vm.com/guide/getting-started.html)
- [`asdf` Introduction](https://asdf-vm.com/guide/introduction.html)
- [`asdf` Versions](https://asdf-vm.com/manage/versions.html)
- [`asdf` Configuration](https://asdf-vm.com/manage/configuration.html)
- [`pyenv` README](https://github.com/pyenv/pyenv)
- [`pyenv-virtualenv` README](https://github.com/pyenv/pyenv-virtualenv)
