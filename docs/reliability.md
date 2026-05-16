# Reliability and Recovery Contracts

This document describes how vex behaves under adverse conditions: network failures,
corrupted downloads, concurrent invocations, partial writes, and abrupt termination.
Every contract here is enforced by code; source references point to the implementation
so you can audit the actual behavior.

## 1. Network retries

**Policy:** Up to `N` retries with a fixed delay between attempts.
The implementation does **not** use exponential backoff — each retry waits the same
configured base delay.

| Setting | Default | Source |
|---------|---------|--------|
| `MAX_DOWNLOAD_RETRIES` | 3 | `src/config/model.rs:17` |
| `RETRY_BASE_DELAY` | 1 second | `src/config/model.rs:20` |
| Override per project | `[network] download_retries = N` in `.vex.toml` | `src/project.rs:22` |

A transient failure (5xx, connection reset, DNS timeout) triggers a retry.
A permanent failure (404, 403) does **not** retry — vex returns immediately with
`VexError::Network` (`src/error.rs:17`).

## 2. Checksum verification

vex verifies SHA-256 on every archive before extraction.

**Sources** (each tool resolves its own canonical checksum):

- Node: `https://nodejs.org/dist/v$VERSION/SHASUMS256.txt`
- Go: `sha256` field embedded in the go.dev JSON API response
- Java: `binary.package.checksum` from the Adoptium Temurin API
- Rust: `.sha256` sidecar for every artifact in the channel manifest
- Python: `SHA256SUMS` published with each python-build-standalone release

**Failure modes:**

1. Checksum mismatch → `VexError::ChecksumMismatch { expected, actual }`
   (`src/checksum.rs`). The downloaded archive is discarded; no extraction occurs.
2. Checksum source unreachable → install **refuses to proceed**
   (`src/installer/online.rs:91-95`). The error message reads:
   `"Failed to fetch checksum for verification: …. Refusing to install unverified binary."`
3. Cached archive checksum lost or mismatched → cache entry is invalidated and the
   archive is re-downloaded.

There is no "skip checksum" flag. Adding one would require an explicit configuration
change and is intentionally not exposed.

## 3. Offline mode

`vex install --offline` (or `[network] offline = true` in settings):

- Must find a cached archive **and** its stored checksum in the archive cache
  (`~/.vex/cache/archives/`).
- Cache miss → returns `VexError::OfflineModeError` immediately. vex does not fall
  back to network.
- Cache hit but checksum mismatch → treated as corruption; the archive is removed
  and the operation fails (in offline mode, vex cannot re-download).

Implementation: `src/installer/offline.rs`.

## 4. CleanupGuard (RAII for temp files)

Every install / extract path constructs a `CleanupGuard` that owns the list of
temporary paths it created (`src/installer/support.rs`).

- On normal completion the guard's tracked paths are removed.
- On panic or early return (`?`) the guard's `Drop` impl runs and unlinks every
  tracked path. There are no orphaned temp directories under `~/.vex/cache/`.
- Successful installs explicitly `disarm()` the guard so the final installed
  directory is **not** cleaned up.

This means: if a `cargo install vex` SIGKILL happens mid-extract, you may have a
partial archive in cache, but you will never have a half-symlinked toolchain
under `~/.vex/toolchains/`.

## 5. Concurrent install protection

Two `vex install node@20` invocations cannot collide:

- File-based exclusive lock at `~/.vex/locks/<tool>-<version>.lock`
  (`src/lock.rs`, using `fs2::FileExt::try_lock_exclusive`).
- A stale lock (process exited without releasing) is detected by checking the PID
  recorded in the lock file (`src/lock.rs:42-59`). If the PID is gone, the lock is
  reclaimed.
- Different tools/versions install in parallel without contention; only same-tool-same-version
  is serialized.

## 6. Disk space pre-check

Before downloading, vex calls `check_disk_space` (`src/installer/support.rs:11`)
against `MIN_FREE_SPACE_BYTES = 1.5 GB` (`src/config/model.rs:29`).

If the target filesystem has less than 1.5 GB available, install aborts with
`VexError::DiskSpace { need, available }` before any network traffic — no partial
archive, no half-downloaded blob.

Why 1.5 GB and not the actual archive size: some toolchains (Rust, Java) extract to
several hundred megabytes; the 1.5 GB threshold conservatively accommodates the
largest case plus extraction overhead.

## 7. Atomic version switching

`vex use node@20` updates a single symlink: `~/.vex/current/node`.

The implementation (`src/switcher/links.rs`) follows a temp + rename pattern:

1. Create `~/.vex/current/node.tmp-<uuid>` pointing at the new toolchain.
2. `fs::rename(tmp, current)` — atomic on POSIX filesystems.

A crash between step 1 and step 2 leaves the temp symlink dangling (cleaned up by
the next `vex use` or `vex doctor`); the live `current/node` symlink is never in a
broken state.

## 8. Rollback on a bad switch

If `vex use node@<bad-version>` activated a broken toolchain, recover by activating
the previous version:

```bash
vex use node@<previous-version>
```

The symlink switch is itself atomic (see §7), and toolchain directories are never
deleted by `vex use` — only by `vex uninstall` or `vex prune`. So the previous
toolchain remains on disk and can be re-activated.

For team-pinning recovery, `.tool-versions.lock` records the exact version that was
in use; `vex relink` re-applies the lockfile.

## 9. Self-update safety

`vex self-update` (`src/updater.rs`) follows the same temp + rename pattern for the
vex binary itself:

1. Download the new release to a temp path next to the current binary.
2. Verify SHA-256 against the GitHub release checksum file.
3. `fs::rename` over the current binary. POSIX guarantees the old process keeps the
   old binary in memory; the new one is used on next invocation.

A crash mid-download leaves a `.tmp-<uuid>` next to the binary; `vex self-update`
on next run cleans it up before retrying.

## 10. Failure summary

| Scenario | Behavior | Source |
|---|---|---|
| Network 5xx during download | Retry up to `MAX_DOWNLOAD_RETRIES` with fixed delay | `src/downloader/transfer/retry.rs` |
| Network 4xx | No retry; fail with troubleshooting message | `src/error.rs:17` |
| Checksum mismatch | Discard archive; `VexError::ChecksumMismatch` | `src/checksum.rs` |
| Checksum source unreachable | Refuse install | `src/installer/online.rs:91-95` |
| Cached archive corrupted | Invalidate cache; re-download (or fail in offline mode) | `src/archive_cache.rs` |
| `--offline` cache miss | `VexError::OfflineModeError`; no network fallback | `src/installer/offline.rs` |
| Panic / SIGTERM mid-extract | `CleanupGuard` removes temp paths | `src/installer/support.rs` |
| Concurrent same-version install | File lock serializes; PID-based stale-lock recovery | `src/lock.rs` |
| Concurrent different-version install | Run in parallel; no contention | `src/lock.rs` |
| Disk < 1.5 GB | Abort before download | `src/installer/support.rs:11` |
| Bad `vex use` switch | Atomic symlink; re-activate previous version | `src/switcher/links.rs` |
| Crashed self-update | Temp file cleaned on next run; original binary intact | `src/updater.rs` |
