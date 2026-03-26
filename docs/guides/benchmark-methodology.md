# Benchmark Methodology Guide

This guide explains how to benchmark `vex` fairly and how to report results in a way that other people can reproduce.

The short version:

- use `cargo bench` for internal hot paths
- use repeatable CLI measurements for user-facing workflows
- separate warm-cache, cold-cache, and network-dependent scenarios
- never publish a speed claim without the exact commands and environment

## What This Repository Already Benchmarks

The current `benches/benchmarks.rs` file benchmarks these internal operations:

- parsing `.tool-versions`
- resolving versions from nested directories
- resolving all versions from a project tree
- updating symlinks for version switching
- switching multiple tools
- cache write, read, and read-plus-validate cycles
- parallel versus sequential extraction simulation

Those are valuable because they cover the main hot paths inside `vex`, but they are still microbenchmarks. They do not automatically prove end-user experience for complete installs, CI jobs, or cross-tool comparisons.

## Benchmark Categories

Keep benchmark categories separate in your notes and reports.

### 1. Internal microbenchmarks

Use these when you change parsing, switching, caching, or extraction internals.

```bash
cargo bench
cargo bench bench_parse_tool_versions
cargo bench bench_switch_symlinks
```

Best for:

- validating regressions after refactors
- checking whether a narrow optimization actually helped
- comparing one `vex` commit against another

Not enough for:

- proving full install time improvements
- comparing `vex` against other managers end to end

### 2. Local CLI workflow benchmarks

Use these when you want to measure commands users actually run.

Good candidates:

- `vex current`
- `vex exec -- node -v`
- `vex use node@<version>`
- `vex install` from an existing project file after required archives are already cached
- `vex sync --frozen` in a checked-out repo

Recommended tool:

- `hyperfine` if available

Example:

```bash
hyperfine --warmup 3 'vex current'
hyperfine --warmup 3 'vex exec -- node -v'
```

If `hyperfine` is not available, use a repeatable shell loop and capture multiple runs instead of trusting one `time` result.

### 3. Install and network benchmarks

Use these only when the test explicitly cares about download, extraction, or cache behavior.

Always label each run as one of:

- cold cache
- warm archive cache
- warm remote-version cache
- offline

Do not compare a warm-cache `vex` run against a cold-cache competitor without saying so.

### 4. Cross-manager comparisons

Use these when comparing `vex` against `nvm`, `asdf`, or `pyenv`.

Keep comparisons narrow and honest:

- compare Node workflows to Node workflows
- compare Python workflows to Python workflows
- do not use `vex`'s multi-language workflow as if it were a direct substitute for a Node-only benchmark
- do not compare a shim-based command path and a symlink-based command path without explaining that difference

## Reporting Template

Every public benchmark note should include:

- `vex` version or commit SHA
- comparison tool version and setup method
- macOS version
- machine architecture: Apple Silicon or Intel
- shell used for the command
- whether caches were cold or warm
- whether network access was required
- the exact command lines
- number of runs and warmup policy
- raw output or attached logs

A benchmark claim without this context should be treated as anecdotal.

## Recommended Workflow for Internal Changes

If you are changing `vex` internals and want trustworthy data:

1. Run the relevant `cargo bench` target before the change.
2. Run the same benchmark after the change.
3. If the change affects real CLI behavior, add one user-facing workflow benchmark as well.
4. Keep machine state as similar as possible between runs.
5. Record both the median and spread, not just the fastest run.

This prevents a microbenchmark win from being mistaken for a user-visible improvement.

## Recommended Workflow for User-Facing Docs or Blog Posts

If you want to publish results outside the repository:

1. State whether the benchmark is about install time, command latency, shell overhead, or CI setup time.
2. Publish the exact benchmark commands.
3. Separate local-only timings from network-dependent timings.
4. Repeat each command enough times to smooth out background noise.
5. Avoid single-number marketing claims such as "X is 10x faster" unless the scope is extremely narrow and clearly labeled.

Good:

- "`vex current` median latency on this MacBook Pro with warm caches was ..."
- "Switching between two already-installed Node versions took ..."
- "Cold-cache Node install on this network took ..."

Bad:

- "`vex` is faster than every other version manager"
- "Shell startup is instant" with no command, shell, or plugin context

## Fairness Rules for `nvm`, `asdf`, and `pyenv` Comparisons

When comparing against other managers, follow their official setup guidance instead of ad-hoc shortcuts.

Also keep these rules:

- use the same shell for all managers
- use the same machine and architecture
- use the same project directory and version file semantics where possible
- pin comparable tool versions
- report whether the comparison includes shell initialization cost
- call out when one manager depends on plugins or extra activation helpers

Examples:

- compare `node --version` after a documented shell setup, not after manually exporting a one-off path
- compare Python environment restore workflows separately from Python interpreter selection
- compare CI setup workflows separately from interactive shell workflows

## Suggested Benchmark Matrix

For most `vex` work, this matrix is enough:

| Scenario | Suggested command | Notes |
|----------|-------------------|-------|
| Internal parse/regression | `cargo bench bench_parse_tool_versions` | stable microbenchmark |
| Version resolution | `cargo bench bench_resolve_all_versions` | good after resolver changes |
| Switch latency | `cargo bench bench_switch_symlinks` | internal switching path |
| Warm command latency | `hyperfine --warmup 3 'vex current'` | user-facing warm path |
| Process activation | `hyperfine --warmup 3 'vex exec -- node -v'` | includes activation plan build |
| Reproducible project restore | `hyperfine --warmup 1 'vex sync --frozen'` | requires committed lockfile |
| Cold install | document exact `vex install <tool@version>` run | label network and cache state |

## CI Benchmark Advice

CI is useful for catching regressions, but it is noisy for performance claims.

Use CI benchmarks for:

- smoke checking that a benchmark command still runs
- rough before/after sanity checks

Do not use CI benchmarks as your only source for:

- shell latency claims
- install speed claims
- cross-manager comparisons

If you publish CI numbers, say which runner type was used and whether caches were restored.

## Common Mistakes

Avoid these traps:

- benchmarking with different caches and calling it a fair comparison
- timing one run and reporting it as typical
- hiding network dependence inside an install benchmark
- comparing different version files or different tool versions
- reporting microbenchmark improvements as if they were end-user latency
- mixing shell startup cost, manager activation cost, and command execution cost into one unexplained number

## Example Benchmark Notes Block

Use a block like this in issues, PRs, or release notes:

```text
Environment:
- vex commit: <sha>
- machine: Apple Silicon / Intel
- macOS: <version>
- shell: zsh
- cache state: warm archive cache, warm remote cache

Commands:
- cargo bench bench_switch_symlinks
- hyperfine --warmup 3 'vex current'
- hyperfine --warmup 3 'vex exec -- node -v'

Notes:
- compared before and after the same change on the same machine
- no network-dependent commands included
```

## See Also

- [README benchmark section](../../README.md#running-benchmarks)
- [Getting Started Guide](getting-started.md)
- [Best Practices Guide](best-practices.md)
