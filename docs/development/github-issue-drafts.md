# GitHub Issue Drafts

This document contains copy-paste-ready issue drafts for the next `vex` roadmap.

Recommended label set:
- `type:feature`
- `type:docs`
- `type:engineering`
- `type:research`
- `area:install`
- `area:sync`
- `area:doctor`
- `area:tui`
- `area:resolver`
- `area:env`
- `area:cache`
- `area:plugin`
- `area:ecosystem`
- `area:docs`
- `priority:p0`
- `priority:p1`
- `priority:p2`
- `priority:p3`
- `ai-ready`

Suggested milestone grouping:
- `v1.3`: issues 02, 03, 04, 08, 09
- `v1.4`: issues 05, 06, 07, 10, 11
- `v1.5+`: issues 12, 13, 14, 15, 16, 17, 18, 19, 20

## 01. Meta: UX, Workflow, and Ecosystem Roadmap

Suggested labels: `type:feature`, `priority:p0`, `ai-ready`

Title:
`Meta: UX, workflow, and ecosystem roadmap for the next phase of vex`

Issue body:

## Background

`vex` already supports multi-tool version management, `.tool-versions`, project-local `.vex.toml`, health checks, self-update, interactive version selection, and upgrade/outdated flows. The next phase should improve product quality in three directions:

- better decision support, such as lifecycle and upgrade advisories
- better workflows, such as sync, lockfiles, offline mode, and team sharing
- better terminal UX, including richer interactive output and a future full-screen TUI

## Goal

Create a roadmap umbrella issue that tracks the next set of user-facing improvements and links all implementation issues underneath it.

## Scope

Track the following issue groups:

- upgrade intelligence and EOL/security advisories
- batch install and `vex sync`
- rich terminal UI foundation
- `vex tui` dashboard
- user-defined aliases
- tool-specific environment variables
- better version-not-found suggestions
- `vex doctor` expansion
- offline mode and cache improvements
- lockfile and frozen installs
- plugin system
- project templates
- remote/team config sync
- ecosystem integrations
- docs, benchmarks, and engineering quality

## Non-goals

- Do not implement features directly in this meta issue.
- Do not use this issue for detailed design review of a specific command.

## Acceptance criteria

- The issue links every child issue in this document.
- Each child issue has a clear priority and milestone.
- The roadmap is easy for contributors and AI agents to follow.

## Child issues

- #02 Upgrade intelligence and lifecycle advisories
- #03 Multi-spec install and `vex sync`
- #04 Rich terminal UI foundation
- #05 `vex tui` dashboard
- #06 User-defined aliases
- #07 Tool-specific environment variables
- #08 Better version-not-found suggestions
- #09 `vex doctor` 2.0
- #10 Offline mode and archive cache
- #11 Lockfile and frozen installs
- #12 Plugin system
- #13 Project templates
- #14 Remote team config sync
- #15 Official GitHub Action
- #16 Official Docker integration
- #17 VS Code integration
- #18 Docs and benchmark work
- #19 Engineering quality improvements
- #20 Delta download research

## 02. Feature: Upgrade Intelligence and Lifecycle Advisories

Suggested labels: `type:feature`, `area:doctor`, `area:resolver`, `priority:p0`, `ai-ready`

Title:
`Feature: add upgrade intelligence and lifecycle advisories for installed and managed toolchains`

Issue body:

## Background

`vex` can already resolve versions, show outdated status, and upgrade managed tools. However, it currently answers only "is there a newer version?" and not "should the user upgrade now?"

This is most visible for:

- EOL toolchains such as old Node LTS releases
- releases with upcoming LTS end dates
- branches with security-only upgrade recommendations

## Problem

Users need lifecycle guidance, not just latest-version comparisons. A version manager should warn when a currently managed version is no longer a good default.

## Goal

Add lifecycle-aware upgrade intelligence that can be shown during:

- `vex use`
- `vex install`
- `vex outdated`
- `vex doctor`

## Proposed UX

Examples:

```text
$ vex use node@16

warning: node@16.20.2 is end-of-life
recommendation: upgrade to node@20 (current LTS)
```

```text
$ vex outdated

node   16.20.2 -> 20.11.1  (eol)
java   17.0.10 -> 21.0.2   (lts_available)
python 3.12.1  -> 3.12.4   (security_update_available)
```

## Implementation notes

- Introduce a lifecycle/advisory data model, likely in a new module such as `src/advisories.rs`.
- Keep data collection tool-specific at first.
- Start with Node, Java, and Python.
- Reuse current report-producing commands instead of adding ad hoc prints.
- Make the advisory data available to both text and JSON output.
- Avoid hard-coding lifecycle logic inside `main.rs`.

Likely touchpoints:

- `src/commands/updates.rs`
- `src/commands/doctor/`
- `src/tools/node.rs`
- `src/tools/java.rs`
- `src/tools/python.rs`

## Non-goals

- Do not implement automatic background updates.
- Do not add silent network calls on every shell prompt.
- Do not block installs of EOL versions in the first iteration.

## Acceptance criteria

- `vex outdated` includes advisory status beyond simple latest-version comparison.
- `vex doctor` reports EOL or near-EOL managed versions.
- `vex use` and `vex install` can emit a short warning when the selected version is EOL.
- JSON output includes normalized advisory fields.
- Unsupported tools degrade gracefully without errors.

## Tests

- Unit tests for advisory classification logic.
- Command tests for `outdated` and `doctor` output.
- Regression tests to ensure commands still work when advisory data is unavailable.

## 03. Feature: Multi-Spec Install and `vex sync`

Suggested labels: `type:feature`, `area:install`, `area:sync`, `priority:p0`, `ai-ready`

Title:
`Feature: support multi-spec installs and introduce vex sync as the primary environment sync command`

Issue body:

## Background

`vex install` currently accepts a single spec or installs from version files when no spec is provided. The next step is to support explicit batch workflows and make project sync more discoverable.

## Problem

Current usage is awkward for common project setup flows:

- installing multiple tools requires multiple commands
- "install from project definition" exists, but is not obvious
- batch installs should be able to reuse existing parallel download infrastructure

## Goal

Support both of the following:

```text
vex install node@20 go@1.22 python@3.12
vex sync
```

Also support:

```text
vex install --from .tool-versions
vex sync --from .tool-versions
```

## Proposed UX

- `vex install <spec>...` installs multiple tool specs in one invocation.
- `vex sync` reads the current project/global version definitions and installs missing toolchains.
- `vex sync --use` optionally activates resolved versions after installation.
- Text output should show a summary at the end with installed, skipped, failed counts.

## Implementation notes

- Change CLI parsing so `install` can accept multiple specs.
- Introduce a dedicated sync command instead of hiding sync behavior behind empty `install`.
- Reuse `resolver` for version-file discovery.
- Reuse or extend parallel download support for batch installs.
- Preserve lock safety and atomic install behavior.

Likely touchpoints:

- `src/main.rs`
- `src/installer.rs`
- `src/downloader.rs`
- `src/resolver.rs`
- `src/config.rs`

## Non-goals

- Do not change the on-disk `.tool-versions` format.
- Do not add remote URL sync in this issue.
- Do not implement automatic uninstall/prune behavior here.

## Acceptance criteria

- `vex install node@20 go@1.22` succeeds and installs both requested tools.
- `vex sync` installs missing versions from the current managed context.
- Existing `vex install` single-spec behavior remains compatible.
- Batch installs provide clear per-tool status and a final summary.
- Failure of one tool does not corrupt already completed installs.

## Tests

- CLI argument parsing tests.
- Integration tests for multiple specs.
- Tests for sync from `.tool-versions`.
- Tests for partial failure and summary output.

## 04. Feature: Rich Terminal UI Foundation

Suggested labels: `type:feature`, `area:tui`, `priority:p0`, `ai-ready`

Title:
`Feature: build a rich terminal UI foundation for progress, tables, and interactive command flows`

Issue body:

## Background

`vex` currently mixes plain `println!`, spinner output, progress bars, and one-off interactive selection. The result works, but the UX is inconsistent and hard to scale as more workflows become interactive.

## Problem

The command experience is not visually cohesive across:

- install
- list/list-remote
- current
- outdated/upgrade
- doctor
- prune

## Goal

Introduce a shared terminal UI layer that standardizes:

- step headers
- success/warn/error badges
- aligned tables or lists
- progress rendering
- confirmation prompts
- summary blocks

## Proposed UX

- Installation should look like a multi-step flow.
- Reports such as `doctor`, `current`, and `outdated` should use aligned sections and consistent status markers.
- Interactive prompts should share one visual style.
- Text mode should still work in CI and non-interactive shells.

## Implementation notes

- Create a small internal UI module rather than scattering styling calls.
- Separate data collection from rendering.
- Support at least `plain`, `rich`, and `json` rendering modes internally, even if only text/json are exposed at first.
- Centralize color and icon policy so commands stop formatting independently.

Likely touchpoints:

- `src/output.rs`
- new `src/ui.rs` or `src/render/`
- `src/commands/current.rs`
- `src/commands/updates.rs`
- `src/commands/doctor/render.rs`
- `src/commands/prune.rs`
- `src/installer.rs`

## Non-goals

- Do not implement a full-screen TUI in this issue.
- Do not rewrite all commands at once if incremental migration is cleaner.

## Acceptance criteria

- At least `install`, `doctor`, `current`, and `outdated` use shared rendering primitives.
- Output remains readable in plain terminals.
- Non-interactive mode remains supported.
- JSON output remains unchanged.

## Tests

- Snapshot tests for major text renderers where practical.
- Regression tests for non-interactive behavior.

## 05. Feature: `vex tui` Dashboard

Suggested labels: `type:feature`, `area:tui`, `priority:p1`, `ai-ready`

Title:
`Feature: add vex tui dashboard for current versions, health warnings, and quick actions`

Issue body:

## Background

Once a shared terminal UI foundation exists, `vex` can benefit from a dedicated full-screen entry point for high-signal environment management.

## Problem

Common tasks are currently spread across separate commands:

- checking current versions
- seeing outdated/EOL versions
- installing missing toolchains
- pruning old installs

## Goal

Add a full-screen `vex tui` command that acts as a command center.

## Proposed UX

The initial dashboard should show:

- current active versions
- project-managed versions
- missing installs
- outdated or EOL warnings
- disk usage summary
- quick actions such as install, use, upgrade, prune, and doctor

## Implementation notes

- Use a full-screen terminal UI library only after issue 04 lands.
- Keep the first release read-mostly with a few safe actions.
- Structure the TUI around existing report collectors instead of duplicating business logic.
- Make state refresh explicit and predictable.

## Non-goals

- Do not implement every command inside the TUI on day one.
- Do not replace traditional CLI commands.

## Acceptance criteria

- `vex tui` launches a dashboard successfully in an interactive terminal.
- It shows current versions, warnings, and available actions.
- It can trigger at least one quick action end-to-end.
- It exits cleanly and gracefully declines in non-interactive environments.

## Tests

- Unit tests for dashboard state assembly.
- Smoke tests for TUI command entry conditions.

## 06. Feature: User-Defined Aliases

Suggested labels: `type:feature`, `area:resolver`, `priority:p1`, `ai-ready`

Title:
`Feature: add user-defined aliases with project and global scopes`

Issue body:

## Background

`vex` already supports built-in aliases such as `latest`, `lts`, and `stable`. Users also need named aliases for recurring toolchain choices such as `node-prod` or `python-ci`.

## Problem

Without user-defined aliases, teams either duplicate full versions everywhere or rely on informal naming outside the tool.

## Goal

Add custom aliases with at least two scopes:

- global aliases
- project aliases

## Proposed UX

Examples:

```text
vex alias set node-prod node@20.11.0
vex alias set node-dev node@21.0.0
vex use node@node-prod
vex install node@node-dev
vex alias list
```

Project aliases should be committed with the project config.

## Implementation notes

- Do not overload `.tool-versions` syntax in the first version.
- Store aliases in a dedicated config location, such as:
  - global: `~/.vex/aliases.toml`
  - project: `.vex.toml`
- Alias resolution should happen before fuzzy remote resolution.
- Keep built-in alias handling intact.

## Non-goals

- Do not add shell alias generation.
- Do not support recursive aliases.

## Acceptance criteria

- Users can create, list, and remove aliases.
- `install`, `use`, and other version-consuming commands can resolve user aliases.
- Project aliases override global aliases when names collide.
- Error messages clearly distinguish "unknown alias" from "unknown version".

## Tests

- Alias config parsing tests.
- Resolution precedence tests.
- Command tests for alias use in `install` and `use`.

## 07. Feature: Tool-Specific Environment Variables

Suggested labels: `type:feature`, `area:env`, `priority:p1`, `ai-ready`

Title:
`Feature: auto-export tool-specific environment variables and clarify project environment policy`

Issue body:

## Background

`vex` already supports project-defined environment variables through `.vex.toml`. However, some toolchains also need generated environment variables such as `JAVA_HOME`.

## Problem

Users should not have to manually derive common environment variables from the currently selected toolchain.

## Goal

Add automatic tool-specific environment variables for supported tools, starting with:

- `JAVA_HOME`
- optional future support for language-specific helper variables where appropriate

Also clarify where custom project env belongs.

## Proposed UX

Examples:

```text
vex use java@21
echo $JAVA_HOME
```

Project env remains defined in `.vex.toml`, for example:

```toml
[env]
NODE_OPTIONS = "--max-old-space-size=4096"
GOPROXY = "https://goproxy.cn"
```

## Implementation notes

- Extend activation planning so generated env vars are produced alongside PATH updates.
- Generated env should be deterministic and tool-specific.
- Document that custom env stays in `.vex.toml`, not `.tool-versions`.
- Consider a trait hook on `Tool` for activation env contributions.

Likely touchpoints:

- `src/activation.rs`
- `src/tools/mod.rs`
- `src/tools/java.rs`
- `src/project.rs`
- shell integration output

## Non-goals

- Do not add arbitrary `env` lines to `.tool-versions`.
- Do not add tool-specific env vars that are speculative or unnecessary.

## Acceptance criteria

- `JAVA_HOME` is available when Java is active through `vex`.
- Existing project `[env]` support continues to work.
- Generated env and project env merge predictably.
- Documentation explains the policy clearly.

## Tests

- Activation plan tests for `JAVA_HOME`.
- Merge-order tests for generated env vs project env.

## 08. Feature: Better Version-Not-Found Suggestions

Suggested labels: `type:feature`, `area:resolver`, `priority:p0`, `ai-ready`

Title:
`Feature: improve version resolution errors with suggestions and recovery hints`

Issue body:

## Background

Current version resolution errors are serviceable but generic. This is a high-leverage UX area because many first-time failures happen during version selection.

## Problem

When a user requests a non-existent version, `vex` should suggest likely intended versions instead of only returning "version not found".

## Goal

Add structured, actionable suggestions to version-resolution failures.

## Proposed UX

Example:

```text
Version 'node@20.99.0' not found

Did you mean:
- node@20.11.0 (latest in 20.x)
- node@20.10.0
- node@21.0.0 (latest)

Run 'vex list-remote node' to see all available versions.
```

## Implementation notes

- Resolve suggestions from the same remote version list used for fuzzy resolution.
- Include at least:
  - latest in the requested major/minor line
  - nearest exact versions if available
  - latest overall
- Keep error construction structured so it can later support JSON.

Likely touchpoints:

- `src/tools/mod.rs`
- `src/error.rs`
- command entry points that surface `VersionNotFound`

## Non-goals

- Do not silently auto-correct a version.
- Do not make extra network requests if the necessary version list is already available.

## Acceptance criteria

- Common invalid version requests show 2 to 3 useful suggestions.
- Suggestions are tool-aware.
- Existing error handling remains readable in non-interactive mode.

## Tests

- Unit tests for suggestion ranking.
- Regression tests for exact versions, aliases, and partial versions.

## 09. Feature: `vex doctor` 2.0

Suggested labels: `type:feature`, `area:doctor`, `priority:p0`, `ai-ready`

Title:
`Feature: expand vex doctor with lifecycle warnings, disk usage, and cleanup guidance`

Issue body:

## Background

`vex doctor` already checks installation state, shell hooks, config, symlinks, cache integrity, and binary health. It is a strong foundation for a more decision-oriented diagnostic command.

## Problem

Users need `doctor` to answer:

- what is risky
- what is wasteful
- what action should I take next

## Goal

Extend `vex doctor` with:

- EOL or near-EOL warnings
- unused version counts
- disk usage summary by tool
- cleanup recommendations
- optional performance measurements if cheap and reliable

## Proposed UX

Example output should include:

- installation health
- shell and PATH health
- lifecycle warnings
- unused versions with reclaimable space
- total disk usage by tool
- clear next-step recommendations

## Implementation notes

- Keep the check/report/render separation already used by `doctor`.
- Add new structured fields to the report so JSON remains first-class.
- Reuse prune logic where possible for "unused version" detection.
- Avoid expensive performance probes unless they are fast and deterministic.

Likely touchpoints:

- `src/commands/doctor/checks.rs`
- `src/commands/doctor/types.rs`
- `src/commands/doctor/render.rs`
- `src/commands/prune.rs`

## Non-goals

- Do not automatically delete anything from `doctor`.
- Do not make `doctor` slow because of broad network activity.

## Acceptance criteria

- `doctor` reports unused toolchains and reclaimable space.
- `doctor` reports lifecycle concerns for managed versions.
- `doctor --json` includes the same signal in machine-readable form.
- Suggested follow-up commands are explicit.

## Tests

- Unit tests for disk usage and unused-version detection.
- Snapshot or structured tests for report rendering.

## 10. Feature: Offline Mode and Archive Cache

Suggested labels: `type:feature`, `area:cache`, `area:install`, `priority:p1`, `ai-ready`

Title:
`Feature: add offline mode, reusable archive cache, and cache policies for installs`

Issue body:

## Background

`vex` already caches remote version lists. It should also support more explicit offline and archive reuse workflows.

## Problem

Users working in CI, travel, or unstable networks need:

- cached version-list reuse
- cached archive reuse
- a clear failure mode when offline data is unavailable

## Goal

Add:

- `--offline` for relevant commands
- optional archive cache retention and reuse
- clear cache policy documentation

## Proposed UX

Examples:

```text
vex install node@20 --offline
vex list-remote node --offline
vex sync --offline
```

Behavior:

- when cached metadata and archives are available, use them
- when required cache entries are missing, fail with an actionable message

## Implementation notes

- Distinguish metadata cache from archive cache.
- Add config knobs for archive retention if needed.
- Keep offline behavior explicit rather than implicit.
- Ensure checksum verification still works with cached archives.

## Non-goals

- Do not invent stale metadata when cache is missing.
- Do not silently fall back to network when `--offline` is requested.

## Acceptance criteria

- Commands fail fast with a clear message when offline data is unavailable.
- Cached archives can satisfy repeated installs without redownloading.
- Offline mode works for at least install, sync, and list-remote.

## Tests

- Tests for cache hit/miss behavior in offline mode.
- Tests for repeated install using archive cache.

## 11. Feature: Lockfile and Frozen Installs

Suggested labels: `type:feature`, `area:sync`, `area:cache`, `priority:p1`, `ai-ready`

Title:
`Feature: add lockfile support and frozen installs for reproducible toolchain setups`

Issue body:

## Background

Projects often need fully reproducible toolchain selection, not just floating aliases or partial versions.

## Problem

`.tool-versions` expresses intent, but not a fully locked, verifiable toolchain state.

## Goal

Add support for a lockfile such as `.tool-versions.lock` and a `--frozen` install mode.

## Proposed UX

Examples:

```text
vex lock
vex install --frozen
vex sync --frozen
```

The lockfile should record exact versions and integrity data where feasible.

## Implementation notes

- Define a stable lockfile format.
- Prefer exact versions plus checksum/digest data where available.
- Frozen mode should refuse to mutate the lockfile or resolve a different version.
- Keep compatibility with existing `.tool-versions` workflows.

## Non-goals

- Do not replace `.tool-versions`.
- Do not add per-platform lockfile fragmentation in the first version unless unavoidable.

## Acceptance criteria

- `vex lock` writes a valid lockfile from the current managed context.
- `install --frozen` and `sync --frozen` honor the lockfile strictly.
- Clear errors are shown when the lockfile and requested state disagree.

## Tests

- Lockfile read/write tests.
- Frozen mode success/failure tests.

## 12. Feature: Plugin System for Custom Tools

Suggested labels: `type:feature`, `area:plugin`, `priority:p2`, `ai-ready`

Title:
`Feature: add a plugin system for user-defined tool sources and installers`

Issue body:

## Background

Today `vex` ships first-party support for a fixed set of tools. A plugin system would let users manage additional tools without waiting for core releases.

## Problem

Users who want to manage tools such as Deno or internal binaries cannot extend `vex` cleanly.

## Goal

Introduce a plugin mechanism for custom tools defined by config files and, if needed later, external executables.

## Proposed UX

Possible structure:

```toml
[tool]
name = "deno"
bin_names = ["deno"]

[versions]
list_url = "..."
download_url = "..."
```

## Implementation notes

- Start with declarative file-based plugins before executable plugins.
- Define plugin storage layout under `~/.vex/plugins/`.
- Validate plugins strictly before use.
- Reuse existing installer, resolver, and activation behavior where possible.

## Non-goals

- Do not execute arbitrary plugin code in the first iteration.
- Do not support every upstream packaging format immediately.

## Acceptance criteria

- Users can register a custom tool through a plugin definition file.
- `install`, `list-remote`, and `use` work for a valid plugin-defined tool.
- Invalid plugin definitions fail with clear validation errors.

## Tests

- Plugin config parsing tests.
- End-to-end tests with a mock plugin fixture.

## 13. Feature: Project Templates

Suggested labels: `type:feature`, `area:sync`, `area:docs`, `priority:p2`, `ai-ready`

Title:
`Feature: add project templates and vex init --template for bootstrapping environments`

Issue body:

## Background

Many new projects want both toolchain setup and starter config in one place.

## Problem

`vex` can provision toolchains but cannot yet help initialize common project layouts.

## Goal

Add template-driven project bootstrap support.

## Proposed UX

Examples:

```text
vex init --template node-typescript
vex init --template go-service
```

Templates can generate:

- `.tool-versions`
- starter config files
- optional `.vex.toml`

## Implementation notes

- Keep template rendering deterministic and auditable.
- Start with built-in templates stored in the repo.
- Make file overwrite behavior explicit.

## Non-goals

- Do not add a remote template marketplace in the first version.

## Acceptance criteria

- Users can list available templates.
- `vex init --template <name>` creates the expected files.
- Existing files are not overwritten silently.

## Tests

- Template rendering tests.
- File conflict behavior tests.

## 14. Feature: Remote Team Config Sync

Suggested labels: `type:feature`, `area:sync`, `priority:p2`, `ai-ready`

Title:
`Feature: support team config sync from remote files and Git repositories`

Issue body:

## Background

Teams may want a centrally maintained `vex` config source for common toolchains or organization defaults.

## Problem

Current sync workflows are local-file only.

## Goal

Add remote sync sources for:

- URL-based config files
- Git repository sources

## Proposed UX

Examples:

```text
vex sync --from https://company.example/vex-config.toml
vex sync --from git@github.com:company/vex-config.git
```

## Implementation notes

- Define a remote config format separate from `.tool-versions` if needed.
- Make trust boundaries explicit.
- Cache remote configs carefully.
- Resolve merge behavior with local project config before implementation.

## Non-goals

- Do not auto-apply remote config on shell startup.
- Do not support arbitrary remote code execution.

## Acceptance criteria

- Users can sync from a supported remote source with explicit opt-in.
- Merge precedence is documented and deterministic.
- Failure modes are explicit and safe.

## Tests

- Parsing and merge tests.
- Remote fetch and cache tests with fixtures or mocks.

## 15. Ecosystem: Official GitHub Action

Suggested labels: `type:feature`, `area:ecosystem`, `priority:p2`, `ai-ready`

Title:
`Ecosystem: publish an official setup-vex GitHub Action for CI workflows`

Issue body:

## Background

An official setup action would reduce friction for GitHub Actions users and make `vex` easier to adopt in CI.

## Goal

Create an official action, tentatively `vex-sh/setup-vex`, with support for:

- explicit tool specs
- auto-install from project definitions
- optional cache reuse

## Proposed UX

Examples:

```yaml
- uses: vex-sh/setup-vex@v1
  with:
    tools: node@20 go@1.22
```

```yaml
- uses: vex-sh/setup-vex@v1
  with:
    auto-install: true
```

## Non-goals

- Do not block on support for every CI provider.

## Acceptance criteria

- The action can install `vex`.
- The action can install requested toolchains.
- The action is documented and versioned.

## Deliverables

- action repository or subdirectory implementation
- README examples
- sample CI workflow in this repo

## 16. Ecosystem: Official Docker Integration

Suggested labels: `type:feature`, `area:ecosystem`, `priority:p2`, `ai-ready`

Title:
`Ecosystem: publish official Docker images and Docker-based vex workflows`

Issue body:

## Background

Docker support would help teams use `vex` in CI, devcontainers, and reproducible local environments.

## Goal

Provide official images and docs for Docker usage.

## Proposed scope

- `vex/base`
- examples for copying `.tool-versions` and running `vex sync`
- optional language-focused images later

## Non-goals

- Do not commit to maintaining many specialized images in the first release.

## Acceptance criteria

- A base image exists and can run `vex sync`.
- Docker usage is documented.
- At least one example Dockerfile is provided.

## 17. Ecosystem: VS Code Integration

Suggested labels: `type:feature`, `area:ecosystem`, `priority:p3`, `ai-ready`

Title:
`Ecosystem: scaffold VS Code integration for .tool-versions detection and quick actions`

Issue body:

## Background

Editor integration can reduce setup friction and make project state more visible.

## Goal

Define and scaffold a VS Code extension that can:

- detect `.tool-versions`
- prompt to install missing tools
- surface current toolchain state

## Non-goals

- Do not build a full IDE platform in the first iteration.

## Acceptance criteria

- Extension scope is documented.
- Repository structure and basic command wiring are in place.
- At least one end-to-end quick action works.

## 18. Docs: Migration Guides, Benchmarks, and Best Practices

Suggested labels: `type:docs`, `area:docs`, `priority:p2`, `ai-ready`

Title:
`Docs: add migration guides, benchmark command, and best-practice documentation`

Issue body:

## Background

Product UX is not only code. Good migration and operational docs will directly affect adoption.

## Goal

Produce a docs package that includes:

- migration guides from `nvm`, `asdf`, and `pyenv`
- a documented `vex benchmark` command or benchmark script
- best-practice guides for teams, monorepos, CI, and troubleshooting

## Proposed deliverables

- command mapping tables
- config translation examples
- benchmark methodology and reproducibility notes
- team workflow recommendations

## Non-goals

- Do not publish benchmark claims without reproducible methodology.

## Acceptance criteria

- New docs live under `docs/`.
- README links to the new guides.
- Benchmark guidance is reproducible and auditable.

## 19. Engineering: Coverage and Failure-Recovery Improvements

Suggested labels: `type:engineering`, `priority:p2`, `ai-ready`

Title:
`Engineering: raise test coverage and harden failure recovery for install and switch workflows`

Issue body:

## Background

As feature count grows, the cost of regressions grows with it. `vex` needs stronger coverage around edge cases and recovery paths.

## Goal

Improve engineering quality in two directions:

- higher confidence through targeted tests
- stronger recovery/cleanup behavior on partial failures

## Proposed scope

- boundary-case tests
- concurrent install tests
- network failure recovery tests
- low-disk-space tests
- cleanup on partial install failure
- symlink conflict repair paths where safe

## Non-goals

- Do not chase a coverage number without improving meaningful risk areas.

## Acceptance criteria

- Core modules have materially better coverage.
- New or existing failure paths are tested.
- Cleanup behavior is deterministic after partial failures.

## 20. Research: Evaluate Delta Downloads for Patch Upgrades

Suggested labels: `type:research`, `priority:p3`, `ai-ready`

Title:
`Research: evaluate whether delta downloads are viable for patch-level tool upgrades`

Issue body:

## Background

Delta downloads sound attractive, but support depends heavily on upstream distribution formats and release infrastructure.

## Goal

Produce a short design note answering whether patch-level delta downloads are realistic for `vex`.

## Research questions

- Which upstreams expose suitable delta artifacts, if any?
- Can integrity verification stay simple and trustworthy?
- Is the complexity justified relative to archive caching?

## Deliverable

- A design note in `docs/development/` with a go/no-go recommendation.

## Non-goals

- Do not implement delta downloads in this issue.

## Acceptance criteria

- The design note compares at least Node, Go, Java, Rust, and Python.
- The conclusion recommends either implementation or deferral with reasons.
