# GitHub Issues: Paste-Ready

This file is intentionally short and copy-friendly.

How to use:

1. Open GitHub issue creation.
2. Choose `Blank issue`.
3. Copy the `Title`.
4. Set the `Labels`.
5. Set the `Milestone`.
6. Copy the `Body` starting from `## Summary`.

## 2026-03-19 Roadmap Update

Before pasting any of the older issue bodies below, update them to match the current scopes:

- templates: five built-in core templates, `--list-templates`, `--dry-run`, and safe `--add-only`
- team config: `vex-config.toml` with `version = 1` plus `[tools]` only, loaded from local files, HTTPS, or Git, with local `.tool-versions` taking precedence
- GitHub Action: repository-root macOS-only composite action with cache restore and explicit re-activation
- Docker: deferred, not an active implementation item
- engineering quality: emphasize cleanup, rollback, and failure-recovery validation rather than chasing a coverage percentage
- older file-path references below should be read as responsibility areas, not exact current files; the live codebase now uses `src/main.rs` only as a thin entrypoint and routes through `src/app.rs`, `src/cli/`, and subsystem directories

---

## 01

Title:
`Meta: next-phase UX, workflow, and ecosystem roadmap for vex`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- none

Body:

## Summary

This is a meta issue for the next phase of `vex`.

The goal is to improve:

- upgrade intelligence
- sync and batch workflows
- terminal UX and TUI
- environment management
- offline and lockfile workflows
- ecosystem integrations
- docs and engineering quality

## Purpose

This issue should not implement anything directly.

It should track the child issues and keep the roadmap understandable for both human contributors and AI agents.

## Child issues

- Upgrade intelligence and lifecycle advisories
- Multi-spec install and `vex sync`
- Rich terminal UI foundation
- `vex tui` dashboard
- User-defined aliases
- Tool-specific environment variables
- Better version-not-found suggestions
- `vex doctor` 2.0
- Offline mode and archive cache
- Lockfile and frozen installs
- Plugin system
- Project templates
- Remote team config sync
- Official GitHub Action
- Official Docker integration
- VS Code integration
- Migration guides, benchmarks, and best practices
- Coverage and failure-recovery improvements
- Delta download research

## Acceptance criteria

- All child issues are linked here after creation.
- Milestones are assigned.
- This issue stays updated as the roadmap changes.

---

## 02

Title:
`Feature: add upgrade intelligence and lifecycle advisories for installed and managed toolchains`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.3`

Body:

## Summary

Add lifecycle-aware upgrade intelligence to `vex`.

Today `vex` can tell users whether a newer version exists, but it does not clearly tell them whether the version they are using is already a bad choice, such as:

- end-of-life
- near end-of-life
- behind the current LTS line
- behind a likely security or bugfix recommendation

## Problem

Users can keep using old toolchains without realizing they are no longer recommended.

Examples:

- `node@16` is already EOL
- an older Java LTS may still work but a newer LTS is a better default
- a Python branch may need a newer bugfix/security release

## Scope

Add lifecycle and advisory logic that can be surfaced in:

- `vex install`
- `vex use`
- `vex outdated`
- `vex doctor`

Start with:

- Node.js
- Java
- Python

## Proposed UX

Example:

    $ vex use node@16

    warning: node@16.20.2 is end-of-life
    recommendation: upgrade to node@20 (current LTS)

Example:

    $ vex outdated

    node   16.20.2 -> 20.11.1  (eol)
    java   17.0.10 -> 21.0.2   (lts_available)
    python 3.12.1  -> 3.12.4   (security_update_available)

## Non-goals

- Do not block installation of old versions.
- Do not add background auto-update behavior.
- Do not add shell-startup network calls.
- Do not try to solve every tool perfectly in the first version.

## Notes for implementation

Likely touchpoints:

- `src/commands/updates.rs`
- `src/commands/doctor/`
- `src/tools/node.rs`
- `src/tools/java.rs`
- `src/tools/python.rs`

It may be useful to add a small advisory module such as `src/advisories.rs`.

## Acceptance criteria

- `vex outdated` can show lifecycle/advisory status.
- `vex doctor` warns for EOL or near-EOL managed versions.
- `vex install` and `vex use` can print short lifecycle warnings.
- Unsupported tools degrade gracefully.
- JSON output includes advisory information where applicable.

## Tests

- Add unit tests for advisory classification.
- Add command tests for `outdated` and `doctor`.
- Add graceful-degradation tests for missing advisory data.

---

## 03

Title:
`Feature: support multi-spec installs and introduce vex sync as the primary environment sync command`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.3`

Body:

## Summary

Support explicit batch installs and add a dedicated `vex sync` command.

## Problem

Current project setup is less ergonomic than it should be:

- multiple tool installs require multiple commands
- installing from version files is possible but not very discoverable
- a dedicated sync workflow does not exist yet

## Scope

Support:

    vex install node@20 go@1.22 python@3.12

Support:

    vex sync

Also support:

    vex install --from .tool-versions
    vex sync --from .tool-versions

## Non-goals

- Do not change the `.tool-versions` file format.
- Do not add remote config sync in this issue.
- Do not automatically uninstall anything.

## Notes for implementation

Likely touchpoints:

- `src/main.rs`
- `src/installer.rs`
- `src/downloader.rs`
- `src/resolver.rs`

Batch installs should reuse existing download and lock protections where possible.

## Acceptance criteria

- `vex install node@20 go@1.22` works in one command.
- `vex sync` installs missing versions from the current managed context.
- Existing single-spec install behavior stays compatible.
- Output shows installed, skipped, and failed items clearly.
- Partial failures do not corrupt successful installs.

## Tests

- Add CLI parsing tests.
- Add integration tests for multi-spec install.
- Add tests for sync from `.tool-versions`.
- Add tests for partial failure behavior.

---

## 04

Title:
`Feature: build a rich terminal UI foundation for progress, tables, and interactive command flows`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.3`

Body:

## Summary

Create a shared terminal UI layer for `vex`.

## Problem

Current output is a mix of:

- plain `println!`
- spinners
- progress bars
- one-off interactive prompts

It works, but the UX is inconsistent and hard to scale.

## Scope

Introduce shared rendering primitives for:

- headers
- success/warn/error states
- tables and aligned rows
- progress steps
- summaries
- interactive prompts

Apply the new layer to at least:

- `install`
- `doctor`
- `current`
- `outdated`

## Non-goals

- Do not build a full-screen TUI in this issue.
- Do not rewrite every command at once if incremental migration is cleaner.

## Notes for implementation

Likely touchpoints:

- `src/output.rs`
- a new `src/ui.rs` or `src/render/` module
- `src/installer.rs`
- `src/commands/current.rs`
- `src/commands/updates.rs`
- `src/commands/doctor/render.rs`

Separate data collection from rendering as much as possible.

## Acceptance criteria

- The listed commands use shared rendering primitives.
- Output is more consistent across commands.
- Non-interactive mode still works.
- JSON output stays unchanged.

## Tests

- Add renderer tests or snapshots where practical.
- Add non-interactive regression tests.

---

## 05

Title:
`Feature: add vex tui dashboard for current versions, health warnings, and quick actions`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.4`

Body:

## Summary

Add a full-screen `vex tui` dashboard.

## Problem

Important state is currently spread across multiple commands:

- current versions
- missing installs
- outdated versions
- EOL warnings
- disk usage

## Scope

Add `vex tui` with an initial dashboard that shows:

- current active versions
- managed versions from project/global context
- missing installs
- outdated or EOL warnings
- disk usage summary
- quick actions for common tasks

## Non-goals

- Do not move every CLI command into the TUI.
- Do not replace the regular CLI.

## Notes for implementation

This should build on the rendering foundation from issue 04.

The first version can be read-mostly with limited safe actions.

## Acceptance criteria

- `vex tui` launches in an interactive terminal.
- It shows current versions and warnings.
- It supports at least one quick action end-to-end.
- It exits cleanly in unsupported environments.

## Tests

- Add state assembly tests.
- Add smoke tests for TUI entry conditions.

---

## 06

Title:
`Feature: add user-defined aliases with project and global scopes`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.4`

Body:

## Summary

Add user-defined aliases on top of built-in aliases like `latest`, `lts`, and `stable`.

## Problem

Teams often want reusable names such as:

- `node-prod`
- `node-dev`
- `python-ci`

Today `vex` only supports built-in aliases.

## Scope

Support:

- global aliases
- project aliases
- alias resolution in `install` and `use`

Example:

    vex alias set node-prod node@20.11.0
    vex alias set node-dev node@21.0.0
    vex use node@node-prod

## Non-goals

- Do not overload `.tool-versions` syntax in the first version.
- Do not support recursive aliases.

## Notes for implementation

A dedicated config file is preferred over changing `.tool-versions`.

Suggested locations:

- global: `~/.vex/aliases.toml`
- project: `.vex.toml`

## Acceptance criteria

- Users can create, list, and remove aliases.
- `install` and `use` can resolve them.
- Project aliases override global aliases when names collide.
- Errors clearly distinguish missing alias from missing version.

## Tests

- Add alias config parsing tests.
- Add precedence tests.
- Add command tests for alias usage.

---

## 07

Title:
`Feature: auto-export tool-specific environment variables and clarify project environment policy`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.4`

Body:

## Summary

Automatically provide tool-specific environment variables such as `JAVA_HOME`, while keeping user-defined project env in a clear config location.

## Problem

Some toolchains require conventional environment variables. Users should not need to derive them manually after switching versions.

## Scope

Start with:

- `JAVA_HOME`

Keep project-defined environment variables in `.vex.toml`.

Example:

```toml
[env]
NODE_OPTIONS = "--max-old-space-size=4096"
GOPROXY = "https://goproxy.cn"
```

## Non-goals

- Do not add arbitrary `env` lines to `.tool-versions`.
- Do not add unnecessary generated variables for every tool.

## Notes for implementation

Likely touchpoints:

- `src/activation.rs`
- `src/tools/mod.rs`
- `src/tools/java.rs`
- shell integration output

A tool hook for activation env may be useful.

## Acceptance criteria

- `JAVA_HOME` is set when Java is active through `vex`.
- Existing project `[env]` support continues to work.
- Merge behavior is predictable and documented.

## Tests

- Add activation-plan tests for `JAVA_HOME`.
- Add env merge-order tests.

---

## 08

Title:
`Feature: improve version resolution errors with suggestions and recovery hints`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.3`

Body:

## Summary

Improve `Version not found` errors so they help users recover quickly.

## Problem

If a user requests a non-existent version, `vex` should suggest likely intended versions instead of only reporting failure.

## Scope

When version resolution fails, suggest:

- the latest version in the requested major or minor line
- a nearby exact version if useful
- the latest overall version

Example:

    Version 'node@20.99.0' not found

    Did you mean:
    - node@20.11.0 (latest in 20.x)
    - node@20.10.0
    - node@21.0.0 (latest)

    Run 'vex list-remote node' to see all available versions.

## Non-goals

- Do not silently auto-correct user input.
- Do not add unnecessary extra network calls if version data is already available.

## Notes for implementation

Likely touchpoints:

- `src/tools/mod.rs`
- `src/error.rs`

## Acceptance criteria

- Common invalid version inputs show useful suggestions.
- Suggestions are tool-aware.
- The output stays readable in non-interactive mode.

## Tests

- Add suggestion-ranking tests.
- Add regression tests for exact, alias, and partial inputs.

---

## 09

Title:
`Feature: expand vex doctor with lifecycle warnings, disk usage, and cleanup guidance`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.3`

Body:

## Summary

Upgrade `vex doctor` from a pure health checker into a more actionable diagnostics command.

## Problem

Users also want `doctor` to tell them:

- which installed versions are risky
- which ones are unused
- how much disk space they consume
- what action to take next

## Scope

Add to `vex doctor`:

- EOL and near-EOL warnings
- unused version counts
- reclaimable disk space
- disk usage grouped by tool
- follow-up command suggestions

## Non-goals

- Do not automatically delete anything.
- Do not make `doctor` slow with unnecessary network work.

## Notes for implementation

Likely touchpoints:

- `src/commands/doctor/checks.rs`
- `src/commands/doctor/types.rs`
- `src/commands/doctor/render.rs`
- `src/commands/prune.rs`

## Acceptance criteria

- `doctor` reports unused versions and reclaimable space.
- `doctor` reports lifecycle concerns.
- `doctor --json` exposes the new information.
- Output includes clear recommended commands.

## Tests

- Add disk-usage and unused-version tests.
- Add renderer or report-structure tests.

---

## 10

Title:
`Feature: add offline mode, reusable archive cache, and explicit cache policies`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.4`

Body:

## Summary

Add explicit offline support and improve cache reuse for installs.

## Problem

Users working with unstable or unavailable network access need:

- cached metadata reuse
- cached archive reuse
- clear behavior when required cache is missing

## Scope

Support:

    vex install node@20 --offline
    vex list-remote node --offline
    vex sync --offline

Also support reusable archive cache when files are already downloaded.

## Non-goals

- Do not silently fall back to network when `--offline` is requested.
- Do not fake data when cache is missing.

## Notes for implementation

Separate:

- metadata cache
- archive cache

Keep offline behavior explicit.

## Acceptance criteria

- Commands fail clearly when offline data is unavailable.
- Cached archives can satisfy repeated installs without redownloading.
- Offline mode works for install, sync, and list-remote.

## Tests

- Add offline cache hit/miss tests.
- Add repeated-install archive reuse tests.

---

## 11

Title:
`Feature: add lockfile support and frozen installs for reproducible toolchain setups`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.4`

Body:

## Summary

Add a lockfile and a frozen install mode for reproducible toolchain setups.

## Problem

`.tool-versions` expresses desired versions, but it does not yet provide a fully locked, verifiable install state.

## Scope

Support:

    vex lock
    vex install --frozen
    vex sync --frozen

The lockfile should capture exact versions and integrity data where practical.

## Non-goals

- Do not replace `.tool-versions`.
- Do not overcomplicate the first format with unnecessary platform fragmentation.

## Notes for implementation

Prefer a stable format such as `.tool-versions.lock`.

Frozen mode should refuse to resolve a different version than what is locked.

## Acceptance criteria

- `vex lock` writes a valid lockfile.
- `install --frozen` and `sync --frozen` honor the lock strictly.
- Errors are clear when the requested state does not match the lockfile.

## Tests

- Add lockfile parse/write tests.
- Add frozen-mode success and failure tests.

---

## 12

Title:
`Feature: add a plugin system for user-defined tool sources and installers`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.5+`

Body:

## Summary

Add a plugin system so users can manage tools that are not built into `vex`.

## Problem

Users cannot currently extend `vex` cleanly for tools like Deno or internal binaries.

## Scope

Start with declarative, file-based plugins under a location like:

- `~/.vex/plugins/`

Support:

- custom tool metadata
- remote version listing
- download URL templates
- binary names

## Non-goals

- Do not execute arbitrary plugin code in the first version.
- Do not support every upstream packaging format immediately.

## Acceptance criteria

- Users can define a custom tool through a plugin file.
- `list-remote`, `install`, and `use` work for valid plugins.
- Invalid plugins fail with clear validation errors.

## Tests

- Add plugin config parsing tests.
- Add an end-to-end test with a mock plugin fixture.

---

## 13

Title:
`Feature: add project templates and vex init --template for bootstrapping environments`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.5+`

Body:

## Summary

Add template-driven project bootstrap support.

## Problem

`vex` can manage toolchains, but it cannot yet help create a repeatable project starter layout.

## Scope

Support:

    vex init --template node-typescript
    vex init --template go-service

Templates can create:

- `.tool-versions`
- starter config files
- optional `.vex.toml`

## Non-goals

- Do not build a remote template marketplace in the first version.

## Acceptance criteria

- Users can list available templates.
- `vex init --template <name>` creates the expected files.
- Existing files are not overwritten silently.

## Tests

- Add template rendering tests.
- Add overwrite and conflict tests.

---

## 14

Title:
`Feature: support team config sync from remote files and Git repositories`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.5+`

Body:

## Summary

Support sync from remote team-managed config sources.

## Problem

Current sync workflows are local-only. Some teams want centrally maintained config sources.

## Scope

Support:

    vex sync --from https://company.example/vex-config.toml
    vex sync --from git@github.com:company/vex-config.git

## Non-goals

- Do not auto-apply remote config on shell startup.
- Do not allow arbitrary remote code execution.

## Acceptance criteria

- Remote sync works with explicit opt-in.
- Merge behavior with local config is deterministic and documented.
- Failures are clear and safe.

## Tests

- Add config merge tests.
- Add remote fetch tests using mocks or fixtures.

---

## 15

Title:
`Ecosystem: publish an official repository-root GitHub Action for CI workflows`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.5+`

Body:

## Summary

Create an official GitHub Action for installing `vex` and managed toolchains in CI.

## Scope

Support at least:

    - uses: imnotnoahhh/vex@v1
      with:
        tools: node@20 go@1.22

And:

    - uses: imnotnoahhh/vex@v1
      with:
        auto-install: true

## Non-goals

- Do not block on support for every CI provider.

## Acceptance criteria

- The action can install `vex`.
- The action can install requested toolchains.
- The action is documented and versioned.

## Deliverables

- action implementation
- README usage examples
- sample workflow

---

## 16

Title:
`Ecosystem: publish official Docker images and Docker-based vex workflows`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.5+`

Body:

## Summary

Publish official Docker integration for `vex`.

## Scope

Start with:

- a base image
- docs for copying `.tool-versions`
- docs for running `vex sync` in Docker builds

## Non-goals

- Do not maintain many specialized images in the first release.

## Acceptance criteria

- A base image exists and can run `vex sync`.
- Docker usage is documented.
- At least one example Dockerfile is provided.

---

## 17

Title:
`Ecosystem: scaffold VS Code integration for .tool-versions detection and quick actions`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.5+`

Body:

## Summary

Define and scaffold VS Code integration for common `vex` workflows.

## Scope

Initial goals:

- detect `.tool-versions`
- detect missing toolchains
- offer quick install actions
- surface current toolchain state

## Non-goals

- Do not build a full editor platform in the first version.

## Acceptance criteria

- Extension scope is documented.
- Basic extension structure exists.
- At least one useful quick action works.

---

## 18

Title:
`Docs: add migration guides, benchmark command guidance, and best-practice documentation`

Labels:
- `documentation`
- `help wanted`

Milestone:
- `v1.5+`

Body:

## Summary

Expand documentation to make `vex` easier to adopt and compare with alternatives.

## Scope

Add docs for:

- migrating from `nvm`
- migrating from `asdf`
- migrating from `pyenv`
- benchmark methodology and tooling
- team and monorepo best practices
- CI and troubleshooting guidance

## Non-goals

- Do not publish benchmark claims without reproducible methodology.

## Acceptance criteria

- New docs are added under `docs/`.
- README links to the new guides.
- Benchmark guidance is reproducible and reviewable.

---

## 19

Title:
`Engineering: raise test coverage and harden failure recovery for install and switch workflows`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.5+`

Body:

## Summary

Improve engineering confidence through stronger tests and better failure recovery.

## Scope

Focus on:

- edge-case tests
- concurrent install tests
- network failure recovery tests
- low-disk-space tests
- cleanup after partial failures
- safe handling of symlink conflicts

## Non-goals

- Do not chase a coverage number without improving meaningful risk areas.

## Acceptance criteria

- Core workflows have materially better test coverage.
- Important recovery paths are tested.
- Partial failure cleanup is deterministic.

---

## 20

Title:
`Research: evaluate whether delta downloads are viable for patch-level tool upgrades`

Labels:
- `enhancement`
- `help wanted`

Milestone:
- `v1.5+`

Body:

## Summary

Evaluate whether delta downloads are worth implementing for patch upgrades.

## Problem

Delta downloads sound attractive, but they may not fit upstream release formats or verification requirements.

## Scope

Produce a short design note covering:

- Node
- Go
- Java
- Rust
- Python

Answer:

- do upstreams expose suitable delta artifacts
- can verification stay simple and safe
- is the complexity better than archive caching

## Non-goals

- Do not implement delta downloads in this issue.

## Acceptance criteria

- A design note is added under `docs/development/`.
- The note ends with a clear go/no-go recommendation.
