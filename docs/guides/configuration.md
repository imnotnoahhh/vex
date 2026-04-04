# Configuration Guide

vex has two configuration layers:

- `~/.vex/config.toml` for machine-wide defaults
- `.vex.toml` for project-local behavior, network overrides, environment variables, mirrors, and named tasks

For team-wide version baselines, `vex install --from` and `vex sync --from` also support a dedicated `vex-config.toml` format described below.

`CLI flags` override everything. Environment variables override both global and project file-based configuration. Project config is intended for repo-local behavior, not for replacing `.tool-versions`.

## Global Configuration

Global configuration lives in:

```text
~/.vex/config.toml
```

### Example

```toml
cache_ttl_secs = 300

[network]
connect_timeout_secs = 30
read_timeout_secs = 300
download_retries = 3
retry_base_delay_secs = 1
max_concurrent_downloads = 3
max_http_redirects = 10
proxy = "http://proxy.internal:8080"

[behavior]
auto_switch = true
auto_activate_venv = true
default_shell = "zsh"
non_interactive = false
capture_user_state = true

[strict]
home_hygiene = "warn"
path_conflicts = "warn"

[mirrors]
node = "https://mirror.example.com/nodejs"
rust = "https://mirror.example.com/rust"
```

### Supported Global Keys

#### Top-level

- `cache_ttl_secs`
  - `0` disables the remote-version cache
  - `60..=3600` uses the requested TTL
  - values above `3600` are clamped to `3600`

#### `[network]`

- `connect_timeout_secs`
- `read_timeout_secs`
- `download_retries`
- `retry_base_delay_secs`
- `max_concurrent_downloads`
- `max_http_redirects`
- `proxy`

#### `[behavior]`

- `auto_switch`
- `auto_activate_venv`
- `default_shell`
- `non_interactive`
- `capture_user_state`

#### `[strict]`

- `home_hygiene`
  - `warn` reports legacy home-directory state outside `~/.vex`
  - `enforce` upgrades those findings to issues in `vex doctor`
- `path_conflicts`
  - `warn` reports conflicting PATH and captured-env state
  - `enforce` upgrades those findings to issues in `vex doctor`

#### `[mirrors]`

Each entry rewrites the download host for archive fetches while preserving the upstream path:

```toml
[mirrors]
python = "https://cache.internal/python"
```

## Environment Variable Overrides

The same configuration can be overridden in CI or enterprise environments without editing files:

```bash
export VEX_CACHE_TTL_SECS=0
export VEX_PROXY=http://proxy.internal:8080
export VEX_DOWNLOAD_RETRIES=5
export VEX_NON_INTERACTIVE=1
export VEX_MIRROR_NODE=https://mirror.example.com/nodejs
```

Supported environment variables:

- `VEX_CACHE_TTL_SECS`
- `VEX_CONNECT_TIMEOUT_SECS`
- `VEX_READ_TIMEOUT_SECS`
- `VEX_DOWNLOAD_RETRIES`
- `VEX_RETRY_BASE_DELAY_SECS`
- `VEX_MAX_CONCURRENT_DOWNLOADS`
- `VEX_MAX_HTTP_REDIRECTS`
- `VEX_PROXY`
- `VEX_AUTO_SWITCH`
- `VEX_AUTO_ACTIVATE_VENV`
- `VEX_DEFAULT_SHELL`
- `VEX_NON_INTERACTIVE`
- `VEX_CAPTURE_USER_STATE`
- `VEX_MIRROR_<TOOL>`

## Project Configuration

Project configuration lives in:

```text
.vex.toml
```

It is searched upward from the current working directory, similar to `.tool-versions`.

### Example

```toml
[behavior]
auto_switch = true
auto_activate_venv = true
default_shell = "zsh"
non_interactive = false

[network]
connect_timeout_secs = 10
read_timeout_secs = 120
download_retries = 5
retry_base_delay_secs = 2
proxy = "http://proxy.team.internal:8080"

[mirrors]
node = "https://mirror.example.com/nodejs"
python = "https://mirror.example.com/python"

[env]
RUST_LOG = "debug"
APP_ENV = "dev"

[commands]
test = "cargo test"
lint = "cargo clippy --all-targets --all-features -- -D warnings"
dev = "node server.js"
```

### Recommended Responsibilities

- `.tool-versions`
  - choose tool versions
- `.vex.toml`
  - define project tasks
  - define project env vars
  - adjust project-local behavior
  - tune network settings for that repository
  - point selected tools at repo-specific mirrors

### Team Config Sync (`vex-config.toml`)

Remote and shared team config is intentionally narrower than `.vex.toml`.

Supported sources:

- local `vex-config.toml`
- HTTPS URL pointing to `vex-config.toml`
- HTTPS Git repository with `vex-config.toml` at the repo root
- SSH Git repository with `vex-config.toml` at the repo root

Supported schema:

```toml
version = 1

[tools]
node = "20"
go = "1.24"
java = "21"
rust = "stable"
python = "3.12"
```

Important limits:

- only `[tools]` is supported
- remote team config cannot define `env`, `commands`, mirrors, shell behavior, or arbitrary scripts
- local `.tool-versions` entries override matching tools from team config
- team config is only used when you explicitly pass `--from`
- local `--from` file paths are resolved relative to your current working directory

Examples:

```bash
vex sync --from vex-config.toml
vex sync --from https://company.example/vex-config.toml
vex install --from git@github.com:company/vex-config.git
```

### Supported Project Keys

#### `[behavior]`

- `auto_switch`
- `auto_activate_venv`
- `default_shell`
- `non_interactive`

#### `[network]`

- `connect_timeout_secs`
- `read_timeout_secs`
- `download_retries`
- `retry_base_delay_secs`
- `max_concurrent_downloads`
- `max_http_redirects`
- `proxy`

#### `[mirrors]`

Project mirrors rewrite archive downloads inside that repository only. They are merged with global mirrors, and project entries win over the global file while environment variables still win overall.

## `vex exec`

`vex exec` runs a command in the resolved vex environment without switching global symlinks:

```bash
vex exec -- node -v
vex exec -- python -m pytest
vex exec -- cargo test
```

What it does:

- resolves versions from `.tool-versions` and language-specific files
- prepends the matching toolchain `bin` directories to `PATH`
- prepends the nearest project `.venv/bin` when auto-activation is enabled
- injects captured user-state env vars such as `CARGO_HOME`, `GOPATH`, `NPM_CONFIG_PREFIX`, and `PIP_CACHE_DIR` when enabled
- applies project env vars from `.vex.toml`

## `vex run`

`vex run` executes a named command from `.vex.toml` in the same managed environment:

```bash
vex run test
vex run lint
vex run dev
```

Tasks run from the directory that contains `.vex.toml`, so nested subdirectories still execute from the project root.

## Validation

Use `vex doctor` to validate the active setup:

```bash
vex doctor
vex doctor --json
```

The health check now validates:

- global config readability
- global `tool-versions` syntax
- nearest `.vex.toml` syntax
- PATH presence and priority
- shell hook duplication
- cache integrity
