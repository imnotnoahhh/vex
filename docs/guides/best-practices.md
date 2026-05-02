# Best Practices Guide

This guide collects the practices that make `vex` easier to operate in real projects, especially for teams, monorepos, CI, and Python workflows.

The core rule is simple:

- use version files for tool versions
- use lockfiles for reproducibility
- use `.vex.toml` for project behavior
- keep supported tool home/cache/bin state inside `~/.vex`
- use the shell hook for interactive work and `vex exec` or CI steps for automation

## Keep Responsibilities Separate

Use each file for one job:

- `.tool-versions`
  - checked-in project tool versions
- `.tool-versions.lock`
  - exact reproducible toolchain state with checksums
- `.vex.toml`
  - project tasks, env vars, repo-local behavior, mirrors, and network overrides
- `vex-config.toml`
  - shared team baseline used explicitly with `--from`

Do not overload `.tool-versions` with environment variables, tasks, comments that carry policy, or conventions that only one shell script understands.

## Prefer Exact Versions in Shared Repositories

For shared repositories, prefer exact tool versions instead of floating aliases.

Good:

```text
node 20.11.0
python 3.12.8
```

Riskier for teams:

```text
node lts
python latest
```

Aliases are convenient for local experiments, but exact versions make bug reports, CI failures, and onboarding much easier to reproduce.

## Commit Lockfiles for Reproducibility

If a repository is meant to build the same way across machines, commit the lockfile:

```bash
vex lock
git add .tool-versions .tool-versions.lock
```

Then restore with:

```bash
vex sync --frozen
```

This is the safest default for:

- CI
- release branches
- repositories with multiple contributors
- demos and workshops

## Use `.vex.toml` for Project Behavior

Keep project-local behavior in `.vex.toml`, not in shell-specific startup snippets.

Good uses for `.vex.toml`:

- shared commands via `vex run`
- project env vars
- repo-local mirrors
- stricter retry or timeout settings for one repository

Example:

```toml
[env]
RUST_LOG = "debug"

[commands]
test = "cargo test"
lint = "cargo clippy --all-targets --all-features -- -D warnings"
```

This keeps project behavior visible, reviewable, and portable across shells.

## Team Workflow Recommendations

For most teams, this is the cleanest flow:

1. Commit `.tool-versions`.
2. Commit `.tool-versions.lock` if reproducibility matters.
3. Add shared tasks to `.vex.toml`.
4. Use `vex run` for routine commands.
5. Use `vex outdated` and `vex upgrade` intentionally instead of allowing drift to accumulate.

When you upgrade versions:

```bash
vex outdated
vex upgrade node
vex lock
```

Then review and commit both version files.

## When to Use `vex-config.toml`

Use `vex-config.toml` when a team needs a shared baseline across repositories, but not when a checked-in `.tool-versions` file would already solve the problem.

Good fit:

- internal starter repositories
- centralized baseline recommendations
- org-wide bootstrapping for new repos

Less useful:

- hiding repo-specific choices that should be explicit in the repo itself

Remember:

- `vex-config.toml` is only used when you pass `--from`
- local `.tool-versions` entries override matching tools from that baseline

## Monorepo Patterns

`vex` works best in monorepos when version ownership is easy to read from the directory tree.

### Pattern 1: One shared stack at the repo root

Use one root `.tool-versions` when most projects share the same toolchain.

```text
repo/
  .tool-versions
  .vex.toml
  service-a/
  service-b/
```

This is the simplest setup when every service can use the same Node, Go, Java, Rust, or Python versions.

### Pattern 2: Root defaults plus service overrides

Use a root `.tool-versions` for shared defaults and add child `.tool-versions` files only where a service diverges.

```text
repo/
  .tool-versions
  services/
    legacy-api/
      .tool-versions
    web/
```

This works well because `vex` resolves parent directories and lets child `.tool-versions` entries override matching tools while keeping the parent values for everything else.

### Pattern 3: Per-project tasks in `.vex.toml`

Put `.vex.toml` at the project root that owns the commands.

```text
repo/
  .tool-versions
  services/
    api/
      .vex.toml
    frontend/
      .vex.toml
```

`vex run` executes from the directory that contains the nearest `.vex.toml`, so nested subdirectories still run the correct project command from the correct project root.

## CI Best Practices

For GitHub Actions on macOS, prefer the official action:

```yaml
- uses: imnotnoahhh/vex@v1
  with:
    auto-install: true
```

Or request exact tools:

```yaml
- uses: imnotnoahhh/vex@v1
  with:
    tools: node@20.11.0 go@1.24.0
```

The action:

- installs the published `vex` release
- restores `~/.vex/cache` and `~/.vex/toolchains` when caching is enabled
- re-activates the restored tools so later steps can use `~/.vex/bin`

CI recommendations:

- use either `tools` or `auto-install: true`, not both
- commit `.tool-versions.lock` and run `vex sync --frozen` when reproducibility is important
- prefer `vex exec` or normal workflow steps over interactive shell-hook assumptions
- keep version files in the repository so cache keys reflect real tool changes
- use `vex repair migrate-home` after onboarding to pull supported legacy home state into `~/.vex`

## Node Projects

Install project tools into `node_modules` and commit the package-manager lockfile. When Node is active, `vex` puts the nearest `node_modules/.bin` before managed npm globals in shell hooks, `vex exec`, and `vex run`.

That means direct commands such as `vite`, `eslint`, and `tsc` resolve to the project-installed versions first. Use `npm install -g` for user-level CLIs only; those go into `~/.vex/npm/prefix/bin`.

## Rust Projects

For Rust projects that need official extensions, keep them in `vex` instead of falling back to a second toolchain manager:

```bash
vex rust target add aarch64-apple-ios aarch64-apple-ios-sim
vex rust component add rust-src
```

Example:

```yaml
- uses: imnotnoahhh/vex@v1
  with:
    version: latest

- run: vex sync --frozen

- run: vex exec -- node -v
```

## Local Automation and Scripts

Use the shell hook for interactive development, but use explicit commands for automation.

Good for scripts:

```bash
vex exec -- cargo test
vex exec -- python -m pytest
vex run test
```

Less reliable for scripts:

- assuming an interactive shell hook already ran
- assuming an old shell session has the right tool selected
- calling `vex use` in helper scripts when process-local activation would be enough

Explicit activation makes scripts easier to debug and easier to move into CI later.

## Python Project Recommendations

For Python repositories, `vex` works best with a checked-in interpreter version and a project-local virtual environment.

Recommended flow:

```bash
vex install python@3.12
vex local python@3.12.8
vex python init
vex python freeze
```

Commit:

- `.tool-versions`
- `requirements.lock`

Do not commit:

- `.venv/`

With the shell hook installed, `vex` auto-activates `.venv` when you enter the project and deactivates it when you leave.

Use the Python base environment for user-level CLI tools that are not project dependencies:

```bash
vex use python@3.12
vex python base pip install kaggle
```

That installs into `~/.vex/python/base/<version>`, not into the interpreter toolchain. When no project `.venv` is active, the shell hook exposes the base `bin` directory so commands such as `kaggle` are available. When a project `.venv` is active, `vex` hides the base `bin` directory so global Python CLIs and packages do not affect project dependency resolution.

## Keep PATH Ownership Simple

Version managers are easiest to operate when only one of them owns the front of `PATH`.

Recommended:

- `~/.vex/bin` comes before old manager paths
- old manager init lines are removed after migration

If commands still resolve to old locations:

```bash
which node
which python
hash -r
```

Then run:

```bash
vex doctor
```

## Troubleshooting Discipline

When something feels inconsistent, use the same short checklist every time:

1. Run `vex doctor`.
2. Run `vex current`.
3. Inspect the relevant version file.
4. Check `which <tool>`.
5. Clear shell caches with `hash -r`.
6. Confirm only one version manager is active in your shell startup files.

This catches most migration and PATH problems quickly.

## Suggested Defaults for New Repositories

If you are starting fresh, this is a strong default:

- commit a root `.tool-versions`
- add `.tool-versions.lock` when reproducibility matters
- add `.vex.toml` only when the repo has real shared tasks or env vars
- use exact versions in team repos
- use `vex run` for shared commands
- use the official GitHub Action in CI on macOS

That gives you a setup that is explicit, reviewable, and easy to move between laptops and CI runners.

## See Also

- [Getting Started Guide](getting-started.md)
- [Configuration Guide](configuration.md)
- [Troubleshooting Guide](troubleshooting.md)
- [Migration and Comparison Guide](migration-comparison.md)
- [Benchmark Methodology Guide](benchmark-methodology.md)
