#!/usr/bin/env python3
"""
Run the strict macOS vex validation suite against the local debug build.

This wrapper keeps the validation logic in test_vex_release_strict.py but
switches it into local-build mode, targeting ./target/debug/vex by default.
"""

from __future__ import annotations

import os
import runpy
import shutil
import subprocess
import sys
from pathlib import Path


SKIP_BUILD = os.environ.get("VEX_STRICT_SKIP_BUILD", "").lower() in {"1", "true", "yes"}


def resolve_repo_root() -> Path:
    candidates = []
    env_repo_root = os.environ.get("VEX_REPO_ROOT")
    if env_repo_root:
        candidates.append(Path(env_repo_root).expanduser())
    candidates.extend(
        [
            Path(__file__).resolve().parents[1],
            Path("/Users/qinfuyao/Developer/OpenSource/vex"),
            Path.home() / "Developer" / "OpenSource" / "vex",
        ]
    )

    seen = set()
    for candidate in candidates:
        resolved = candidate.resolve()
        if resolved in seen:
            continue
        seen.add(resolved)
        if (resolved / "Cargo.toml").is_file() and (resolved / "scripts" / "test_vex_release_strict.py").is_file():
            return resolved
    raise SystemExit(
        "could not locate the vex repository; set VEX_REPO_ROOT=/path/to/vex and rerun"
    )


REPO_ROOT = resolve_repo_root()
STRICT_SCRIPT = REPO_ROOT / "scripts" / "test_vex_release_strict.py"
LOCAL_VEX_BIN = REPO_ROOT / "target" / "debug" / "vex"


def resolve_cargo() -> Path | None:
    direct = shutil.which("cargo")
    if direct:
        return Path(direct).resolve()

    candidates = [
        Path.home() / ".cargo" / "bin" / "cargo",
        Path.home() / ".vex" / "bin" / "cargo",
    ]
    test_home = os.environ.get("VEX_TEST_HOME")
    if test_home:
        candidates.append(Path(test_home).expanduser() / ".vex" / "bin" / "cargo")

    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return candidate.resolve()
    return None


def resolve_rust_bin_dirs(cargo: Path) -> list[Path]:
    candidates: list[Path] = []
    path_rustc = shutil.which("rustc")
    if path_rustc:
        candidates.append(Path(path_rustc).resolve().parent)

    test_home = os.environ.get("VEX_TEST_HOME")
    if test_home:
        candidates.append(Path(test_home).expanduser() / ".vex" / "bin")

    try:
        version_root = cargo.resolve().parents[2]
    except IndexError:
        version_root = None
    if version_root is not None:
        candidates.append(version_root / "rustc" / "bin")
        candidates.append(version_root / "cargo" / "bin")

    unique: list[Path] = []
    seen = set()
    for candidate in candidates:
        resolved = candidate.resolve() if candidate.exists() else candidate
        if resolved in seen:
            continue
        seen.add(resolved)
        unique.append(candidate)
    return [candidate for candidate in unique if candidate.is_dir()]


def ensure_local_build() -> None:
    if SKIP_BUILD:
        return
    cargo = resolve_cargo()
    if cargo is None:
        if LOCAL_VEX_BIN.exists():
            print(
                "warning: cargo not found; using the existing target/debug/vex without rebuilding",
                file=sys.stderr,
            )
            return
        raise SystemExit("cargo was not found in PATH and target/debug/vex does not exist")

    print(f"Building local debug vex in {REPO_ROOT} before strict validation...", file=sys.stderr)
    build_env = os.environ.copy()
    current_path = build_env.get("PATH", "")
    rust_bin_dirs = resolve_rust_bin_dirs(cargo)
    build_path_parts = [str(cargo.parent), *(str(path) for path in rust_bin_dirs), current_path]
    build_env["PATH"] = ":".join(part for part in build_path_parts if part)
    completed = subprocess.run(
        [str(cargo), "build"],
        cwd=REPO_ROOT,
        env=build_env,
    )
    if completed.returncode != 0:
        raise SystemExit(completed.returncode)


def main() -> int:
    ensure_local_build()
    os.environ.setdefault("VEX_STRICT_USE_LOCAL_BUILD", "1")
    os.environ.setdefault("VEX_STRICT_VEX_BIN", str(LOCAL_VEX_BIN))
    runpy.run_path(str(STRICT_SCRIPT), run_name="__main__")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
