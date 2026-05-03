# Command Reference

This guide is the release-ready CLI reference for `vex`.

It is based on the current command surface exposed by `vex --help` and the clap definitions in `src/cli/`.

Use it when you want the full command map in one place without jumping between README sections.

## Quick Rules

- Tool specs use the form `tool@version`, for example `node@20`, `go@1.24`, or `python@3.12.8`.
- The built-in tool names are `node`, `go`, `java`, `rust`, and `python`.
- `vex` supports both project-local version files and global defaults in `~/.vex/tool-versions`.
- Commands that support JSON output use `--json`.
- Use `vex help <command>` or `vex <command> --help` for the in-terminal help view.

## Top-Level Commands

```text
vex init
vex install
vex sync
vex use
vex relink
vex list
vex list-remote
vex current
vex globals
vex uninstall
vex env
vex local
vex global
vex lock
vex upgrade
vex outdated
vex prune
vex alias
vex exec
vex run
vex doctor
vex repair
vex self-update
vex tui
vex python
vex rust
```

## Setup and Bootstrapping

### `vex init`

Initialize `~/.vex`, configure shell integration, or bootstrap a project template.

Usage:

```bash
vex init [--shell <shell>]
vex init --list-templates
vex init --template <template>
vex init --template <template> --dry-run
vex init --template <template> --add-only
```

Options:

- `--shell <shell>`
  - valid values: `auto`, `zsh`, `bash`, `fish`, `skip`
- `--template <template>`
  - initialize the current directory with an official template
- `--list-templates`
  - print the available built-in templates
- `--dry-run`
  - preview template changes without writing files
- `--add-only`
  - merge only safe files such as `.tool-versions` and `.gitignore`, then create missing starter files

Examples:

```bash
vex init --shell auto
vex init --list-templates
vex init --template rust-cli
vex init --template python-venv --add-only
```

### `vex env`

Print the generated shell hook for auto-switching.

Usage:

```bash
vex env <shell>
vex env <shell> --exports
```

Arguments:

- `<shell>`
  - `zsh`, `bash`, `fish`, or `nu`

Examples:

```bash
vex env zsh
vex env fish
vex env nu
```

Notes:

- `vex env <shell>` prints the long-lived shell hook you add to your shell config.
- `vex env <shell> --exports` prints the current directory's resolved export/unset block and is primarily used internally by the shell hook.

### `vex doctor`

Run health checks for the current installation.

The report includes core PATH/symlink checks, managed global CLI inventory, Maven/Gradle state, and active PATH conflicts from other tool managers that can shadow vex.

Usage:

```bash
vex doctor
vex doctor --json
vex doctor --verbose
```

Options:

- `--json`
  - print machine-readable diagnostics
- `--verbose`
  - include extra provenance and captured-environment details in text output

### `vex globals`

List global CLIs and build-tool state that can affect command resolution.

Usage:

```bash
vex globals
vex globals --verbose
vex globals npm --json
vex globals pip
vex globals go --json
vex globals cargo
vex globals maven
vex globals mvn
vex globals gradle
```

The inventory includes:

- shared npm globals from `~/.vex/npm/prefix/bin`
- Python base CLIs from `~/.vex/python/base/<version>/bin`
- Python user-base CLIs from `~/.vex/python/user/bin` when installed through pip's official `--user` path
- Go tools from `~/.vex/go/bin`
- Cargo-installed tools from `~/.vex/cargo/bin`
- external `mvn` and `gradle` CLIs found on PATH
- Maven and Gradle build-tool state under `~/.m2` and `~/.gradle`

Each entry includes its path, source kind, and the active vex version source when a matching toolchain is active. For Node, npm globals are a shared vex-managed user-level CLI pool, not a separate prefix per Node version.

Supported filters are `all`, `node`, `npm`, `python`, `pip`, `go`, `rust`, `cargo`, `java`, `maven`, `mvn`, and `gradle`.

### `vex repair`

Preview or apply safe legacy home-directory migrations into `~/.vex`.

Usage:

```bash
vex repair migrate-home
vex repair migrate-home --tool <tool>
vex repair migrate-home --apply
```

Examples:

```bash
vex repair migrate-home
vex repair migrate-home --tool rust
vex repair migrate-home --apply
```

## Tool Installation and Switching

### `vex install`

Install one or more tool specs, or install from version files when no spec is provided.

Usage:

```bash
vex install [spec...]
vex install --from <source>
vex install --frozen
```

Options:

- `--no-switch`
  - install without automatically activating the new version
- `--force`
  - reinstall even if the version already exists
- `--from <source>`
  - install from a version file, `vex-config.toml`, HTTPS URL, or Git repository
- `--frozen`
  - require `.tool-versions.lock` and fail if the lockfile is missing or out of sync
- `--offline`
  - use only cached metadata and archives

Examples:

```bash
vex install node@20
vex install node@20 go@1.24
vex install python@3.12 --no-switch
vex install node@20 --force
vex install --from vex-config.toml
vex install --frozen
vex install node@20 --offline
```

### `vex sync`

Install missing versions from the current managed context.

Usage:

```bash
vex sync
vex sync --from <source>
vex sync --frozen
```

Options:

- `--from <source>`
  - sync from a version file, `vex-config.toml`, HTTPS URL, or Git repository
- `--frozen`
  - strictly enforce `.tool-versions.lock`
- `--offline`
  - use only cached data

Examples:

```bash
vex sync
vex sync --from https://company.example/vex-config.toml
vex sync --frozen
vex sync --offline
```

### `vex use`

Switch the current active version for a tool, or auto-resolve from version files.

Usage:

```bash
vex use <spec>
vex use --auto
```

Options:

- `--auto`
  - read version files such as `.tool-versions`, `.node-version`, or `.python-version`

Examples:

```bash
vex use node@22
vex use python@3.12
vex use --auto
```

### `vex relink`

Rebuild managed binary links for the active toolchain.

Usage:

```bash
vex relink <tool>
```

Notes:

- currently only `node` is supported
- use this only when an executable appears inside the active Node toolchain's `bin`
- shared npm globals installed into `~/.vex/npm/prefix/bin` are already on PATH and do not need relinking
- it only rebuilds links under `~/.vex/bin`; it does not install packages or change shell configuration
- project-local `node_modules/.bin` is preferred automatically when Node is active, so local CLIs win over shared npm globals in shell hooks, `vex exec`, and `vex run`

Examples:

```bash
vex relink node
```

### `vex local`

Write a tool pin into the current directory's `.tool-versions`.

Usage:

```bash
vex local <spec>
```

Example:

```bash
vex local node@20.11.0
```

### `vex global`

Write a global default version into `~/.vex/tool-versions`.

Usage:

```bash
vex global <spec>
```

Example:

```bash
vex global go@1.24
```

### `vex uninstall`

Remove an installed toolchain version.

Usage:

```bash
vex uninstall <spec>
```

Example:

```bash
vex uninstall node@20.11.0
```

## Rust Extensions

### `vex rust`

Manage official Rust targets and components for the active Rust toolchain.

Usage:

```bash
vex rust target list
vex rust target add <name>...
vex rust target remove <name>...
vex rust component list
vex rust component add <name>...
vex rust component remove <name>...
```

Examples:

```bash
vex rust target add aarch64-apple-ios aarch64-apple-ios-sim
vex rust component add rust-src
```

### `vex lock`

Generate `.tool-versions.lock` from the current managed context.

Usage:

```bash
vex lock
```

Example:

```bash
vex lock
```

## Inspection and Discovery

### `vex list`

List locally installed versions for one tool.

Usage:

```bash
vex list <tool>
vex list <tool> --json
```

Example:

```bash
vex list node
vex list python --json
```

### `vex list-remote`

List available upstream versions.

Usage:

```bash
vex list-remote <tool>
vex list-remote <tool> --filter <filter>
```

Options:

- `--filter`, `-f`
  - valid values: `all`, `lts`, `major`, `latest`
- `--no-cache`
  - bypass the remote-version cache
- `--offline`
  - use only cached remote data
- `--json`
  - print machine-readable output

For Python, the `latest` and `major` filters prefer bugfix/security releases over feature or prerelease assets when both are present.

Examples:

```bash
vex list-remote node
vex list-remote node --filter lts
vex list-remote node --filter major --no-cache
vex list-remote python --json
vex list-remote node --offline
```

### `vex current`

Show active versions in the current environment.

Usage:

```bash
vex current
vex current --json
```

## Upgrades, Drift, and Cleanup

### `vex upgrade`

Install and switch to the latest version of one tool, or upgrade the whole managed context.

Usage:

```bash
vex upgrade <tool>
vex upgrade --all
```

Options:

- `--all`
  - upgrade every managed tool in the current context

Examples:

```bash
vex upgrade node
vex upgrade --all
```

### `vex outdated`

Show which managed tools are behind the latest available version.

Usage:

```bash
vex outdated
vex outdated <tool>
vex outdated --json
```

Examples:

```bash
vex outdated
vex outdated python
vex outdated --json
```

### `vex prune`

Remove unused caches, stale locks, and unreferenced toolchains.

Usage:

```bash
vex prune
vex prune --dry-run
```

Options:

- `--dry-run`
  - preview removals without deleting anything

Alias:

- `vex gc`
  - exact alias for `vex prune`

Examples:

```bash
vex prune --dry-run
vex gc
```

## Alias Management

`vex alias` is a command group. There is no `vex alias <tool>` shortcut.

### `vex alias set`

Create a user-defined alias for one tool version.

Usage:

```bash
vex alias set <tool> <alias> <version> [--project]
```

Options:

- `--project`
  - store the alias in `.vex.toml` instead of `~/.vex/aliases.toml`

Examples:

```bash
vex alias set node production 20.11.0
vex alias set node lts-current 20.11.0 --project
```

### `vex alias list`

List aliases globally, per project, or for one tool.

Usage:

```bash
vex alias list
vex alias list [tool]
```

Options:

- `--project`
  - show only project aliases
- `--global`
  - show only global aliases

Examples:

```bash
vex alias list
vex alias list node
vex alias list --project
```

### `vex alias delete`

Remove an alias.

Usage:

```bash
vex alias delete <tool> <alias> [--project]
```

Examples:

```bash
vex alias delete node production
vex alias delete node lts-current --project
```

## Project-Aware Execution

### `vex exec`

Run one command inside the resolved `vex` environment without changing global symlinks.

Usage:

```bash
vex exec -- <command> [args...]
```

Examples:

```bash
vex exec -- node -v
vex exec -- python -m pytest
vex exec -- cargo test
```

### `vex run`

Run a named task from `[commands]` in `.vex.toml`.

Usage:

```bash
vex run <task> [args...]
```

Examples:

```bash
vex run test
vex run lint
vex run dev -- --host 0.0.0.0
```

## Python Workflow Commands

`vex python` currently accepts a single subcommand word rather than nested clap subcommands.

Supported values:

- `init`
  - create `.venv` using the active `vex`-managed Python and record that version in `.tool-versions`
- `freeze`
  - run `pip freeze` and write `requirements.lock`
- `sync`
  - create `.venv` if needed and restore dependencies from `requirements.lock`
- `base`
  - manage the active Python base environment for user-level Python CLIs

`vex python base` accepts these nested forms:

- `vex python base`
  - create the active version's base environment if needed
- `vex python base path`
  - print the active version's base environment path
- `vex python base pip <args...>`
  - run `pip` inside the active version's base environment
- `vex python base freeze`
  - write the base environment package set to `~/.vex/python/base/<version>/requirements.lock`
- `vex python base sync`
  - restore the base environment from that lockfile

Usage:

```bash
vex python base
vex python base pip install kaggle
vex python init
vex python freeze
vex python sync
```

Recommended workflow:

```bash
vex install python@3.12
vex use python@3.12
vex python base pip install kaggle

cd my-project
vex python init
pip install requests flask
vex python freeze
vex python sync
```

The Python base environment is for global CLI tools. Project `.venv` environments stay separate: when the shell hook activates `.venv`, the base environment's `bin` directory is hidden from `PATH` so base-installed tools and packages do not leak into the project.

## Interactive and Self-Management Commands

### `vex tui`

Launch the interactive terminal dashboard.

Usage:

```bash
vex tui
```

Notes:

- requires an interactive terminal
- intended for current-version overview, health warnings, disk usage, and quick actions

### `vex self-update`

Download and install the latest published `vex` release for the current architecture.

Usage:

```bash
vex self-update
```

## Help Commands

Use any of these when you want the built-in CLI help:

```bash
vex --help
vex help
vex help install
vex install --help
vex alias --help
```

## Related Guides

- [Getting Started Guide](getting-started.md)
- [Installation Guide](installation.md)
- [Configuration Guide](configuration.md)
- [Best Practices Guide](best-practices.md)
