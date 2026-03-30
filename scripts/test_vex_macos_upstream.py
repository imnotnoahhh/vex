#!/usr/bin/env python3
"""
Comprehensive macOS validation for vex 1.1.0+.

What it checks:
- Resolves official macOS archives for node/go/java/rust/python from upstream.
- Streams archive contents and derives the exact binaries each tool should expose.
- Installs and activates each tool with vex.
- Verifies toolchain layout, current symlink, ~/.vex/bin symlinks, command resolution,
  and runnable probes.
- Exercises Python venv lifecycle and project-directory auto-switch / auto-activation.

Notes:
- By default this runs against the current HOME and will modify ~/.vex.
- Set VEX_TEST_HOME=/tmp/somewhere to run in an isolated HOME while still using the
  currently installed vex binary.
"""

from __future__ import annotations

import csv
import http.client
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import textwrap
import urllib.error
import urllib.request
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Sequence, Tuple


USER_AGENT = "vex-macos-upstream-test/1.0"
REAL_HOME = Path.home()
TEST_HOME = Path(os.environ.get("VEX_TEST_HOME", str(REAL_HOME))).expanduser().resolve()
VEX_BIN_STR = os.environ.get("VEX_BIN") or shutil.which("vex")
VEX_BIN = Path(VEX_BIN_STR).expanduser().resolve() if VEX_BIN_STR else None
TMP_ROOT = Path(tempfile.mkdtemp(prefix="vex-macos-upstream-"))
PROJECT_ROOT = TMP_ROOT / "project"
LOG_ROOT = TMP_ROOT / "logs"
LOG_ROOT.mkdir(parents=True, exist_ok=True)
FRESH_RUN = os.environ.get("VEX_TEST_REUSE", "").lower() not in {"1", "true", "yes"}


NODE_BIN_RE = re.compile(r"^[^/]+/(bin/[^/]+)$")
GO_BIN_RE = re.compile(r"^[^/]+/(bin/[^/]+)$")
JAVA_BIN_RE = re.compile(r"^[^/]+/(Contents/Home/bin/[^/]+)$")
RUST_BIN_RE = re.compile(
    r"^[^/]+/((?:rustc|cargo|rustfmt-preview|clippy-preview|rust-analyzer-preview)/bin/[^/]+)$"
)
PYTHON_BIN_RE = re.compile(r"^[^/]+/(?:[^/]+/)?(bin/[^/]+)$")
USAGE_RE = r"Usage|用法"


BAD_EXEC_PATTERNS = [
    r"no such file or directory",
    r"exec format error",
    r"bad cpu type",
    r"command not found",
    r"image not found",
    r"library not loaded",
    r"traceback \(most recent call last\)",
]

RUST_WRAPPER_BINS = {"rust-gdb", "rust-gdbgui", "rust-lldb"}
JAVA_STRUCTURAL_BINS = {"jconsole", "jstatd", "rmiregistry"}


class TestFailure(RuntimeError):
    pass


@dataclass
class Probe:
    args: Sequence[str]
    expect: Sequence[str] = field(default_factory=list)
    allow_nonzero: bool = True


@dataclass
class CommandResult:
    cmd: Sequence[str]
    returncode: int
    output: str


@dataclass
class ToolPlan:
    name: str
    display_name: str
    requested_spec: str
    resolved_version: str
    download_url: str
    upstream_bins: Dict[str, str]
    install_spec: str
    bin_regex: re.Pattern[str]
    meta: Dict[str, object] = field(default_factory=dict)

    @property
    def toolchain_root(self) -> Path:
        return TEST_HOME / ".vex" / "toolchains" / self.name / self.resolved_version

    @property
    def cache_archive(self) -> Path:
        return TEST_HOME / ".vex" / "cache" / f"{self.name}-{self.resolved_version}.tar.gz"

    @property
    def audit_archive(self) -> Path:
        return TEST_HOME / ".vex" / "audit-cache" / f"{self.name}-{self.resolved_version}.tar.gz"


class Reporter:
    def __init__(self) -> None:
        self.passed = 0
        self.failed = 0
        self.warned = 0

    def section(self, title: str) -> None:
        print()
        print(f"[ {title} ]")
        sys.stdout.flush()

    def info(self, message: str) -> None:
        print(f"  - {message}")
        sys.stdout.flush()

    def ok(self, message: str) -> None:
        self.passed += 1
        print(f"  ✓ {message}")
        sys.stdout.flush()

    def warn(self, message: str) -> None:
        self.warned += 1
        print(f"  ! {message}")
        sys.stdout.flush()

    def fail(self, message: str) -> None:
        self.failed += 1
        print(f"  ✗ {message}")
        sys.stdout.flush()

    def expect(self, condition: bool, success: str, failure: str) -> None:
        if condition:
            self.ok(success)
        else:
            self.fail(failure)

    def summary(self) -> int:
        print()
        print("============================================================")
        print(f"Passed : {self.passed}")
        print(f"Warned : {self.warned}")
        print(f"Failed : {self.failed}")
        print("============================================================")
        print(f"Test HOME : {TEST_HOME}")
        print(f"Artifacts  : {TMP_ROOT}")
        return 1 if self.failed else 0


REPORT = Reporter()


def main() -> int:
    try:
        verify_prereqs()
        print_banner()

        REPORT.section("Environment")
        REPORT.expect(VEX_BIN.exists(), f"vex binary found at {VEX_BIN}", "vex binary not found in PATH")
        REPORT.expect(
            sys.platform == "darwin",
            "running on macOS",
            f"this script targets macOS but saw {sys.platform}",
        )
        REPORT.info(f"Using TEST_HOME={TEST_HOME}")
        if TEST_HOME == REAL_HOME:
            REPORT.warn("running against the real HOME; ~/.vex and active versions will change")
        else:
            REPORT.ok("isolated HOME mode enabled")
        if FRESH_RUN:
            REPORT.info("Fresh-run mode enabled; previous TEST_HOME toolchains and audit artifacts will be removed")
            reset_test_home()
        else:
            REPORT.warn("reuse mode enabled via VEX_TEST_REUSE; prior TEST_HOME artifacts may be reused")

        vex_version = run_cmd([str(VEX_BIN), "--version"]).output.strip()
        REPORT.expect("vex " in vex_version, f"vex reports version: {vex_version}", f"unexpected vex version output: {vex_version}")

        plans = build_tool_plans()
        alt_node_version = choose_alt_node_version(plans["node"])

        for tool_name in ["node", "go", "java", "rust", "python"]:
            validate_tool(plans[tool_name])

        validate_python_venv(plans["python"])
        validate_project_behavior(plans, alt_node_version)
        validate_doctor()
        return REPORT.summary()
    except KeyboardInterrupt:
        print("\nInterrupted.")
        return 130
    except Exception as exc:
        REPORT.fail(str(exc))
        return REPORT.summary()


def verify_prereqs() -> None:
    if not VEX_BIN:
        raise TestFailure("vex is not in PATH; install or export VEX_BIN first")
    if shutil.which("zsh") is None:
        raise TestFailure("zsh is required for the auto-switch test")


def print_banner() -> None:
    print()
    print("============================================================")
    print("vex macOS upstream validation")
    print("Official archive diff + symlink + runnable + venv + cd hook")
    print("============================================================")


def reset_test_home() -> None:
    vex_home = TEST_HOME / ".vex"
    targets = [
        vex_home / "toolchains",
        vex_home / "current",
        vex_home / "bin",
        vex_home / "cache",
        vex_home / "audit-cache",
        vex_home / "tool-versions",
    ]
    for target in targets:
        if target.is_dir():
            shutil.rmtree(target, ignore_errors=True)
        elif target.exists() or target.is_symlink():
            try:
                target.unlink()
            except FileNotFoundError:
                pass
    vex_home.mkdir(parents=True, exist_ok=True)


def env_for_subprocess() -> Dict[str, str]:
    env = os.environ.copy()
    env["HOME"] = str(TEST_HOME)
    env["PATH"] = f"{TEST_HOME / '.vex' / 'bin'}:{VEX_BIN.parent}:{env.get('PATH', '')}"
    env["CARGO_HOME"] = str(TEST_HOME / ".vex" / "cargo")
    return env


def run_cmd(
    cmd: Sequence[str],
    *,
    cwd: Optional[Path] = None,
    timeout: int = 60,
    allow_nonzero: bool = False,
    env: Optional[Dict[str, str]] = None,
) -> CommandResult:
    completed = subprocess.run(
        list(cmd),
        cwd=str(cwd) if cwd else None,
        env=env or env_for_subprocess(),
        capture_output=True,
        text=True,
        errors="replace",
        timeout=timeout,
    )
    output = (completed.stdout or "") + (completed.stderr or "")
    if not allow_nonzero and completed.returncode != 0:
        raise TestFailure(
            f"command failed ({completed.returncode}): {' '.join(cmd)}\n{output.strip()}"
        )
    return CommandResult(cmd=cmd, returncode=completed.returncode, output=output)


def run_cmd_live(
    cmd: Sequence[str],
    *,
    cwd: Optional[Path] = None,
    timeout: int = 60,
    allow_nonzero: bool = False,
    env: Optional[Dict[str, str]] = None,
) -> CommandResult:
    completed = subprocess.run(
        list(cmd),
        cwd=str(cwd) if cwd else None,
        env=env or env_for_subprocess(),
        timeout=timeout,
    )
    if not allow_nonzero and completed.returncode != 0:
        raise TestFailure(
            f"command failed ({completed.returncode}): {' '.join(cmd)}"
        )
    return CommandResult(cmd=cmd, returncode=completed.returncode, output="")


def fetch_text(url: str, *, headers: Optional[Dict[str, str]] = None) -> str:
    req_headers = {"User-Agent": USER_AGENT}
    if headers:
        req_headers.update(headers)
    last_error: Optional[Exception] = None
    for attempt in range(1, 4):
        request = urllib.request.Request(url, headers=req_headers)
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return read_response_bytes(response).decode("utf-8")
        except http.client.IncompleteRead as exc:
            last_error = exc
            if attempt == 3:
                break
        except (urllib.error.URLError, TimeoutError, http.client.HTTPException) as exc:
            last_error = exc
            if attempt == 3:
                break
        except Exception as exc:
            last_error = exc
            if attempt == 3:
                break
    try:
        REPORT.warn(f"urllib fetch failed for {url}; retrying with curl")
        return fetch_text_with_curl(url, headers=req_headers)
    except Exception as curl_exc:
        raise TestFailure(f"failed to fetch {url}: {last_error}; curl fallback failed: {curl_exc}") from curl_exc


def fetch_json(url: str, *, headers: Optional[Dict[str, str]] = None):
    req_headers = {"User-Agent": USER_AGENT}
    if headers:
        req_headers.update(headers)
    last_error: Optional[Exception] = None
    for attempt in range(1, 4):
        try:
            return json.loads(fetch_text(url, headers=req_headers))
        except json.JSONDecodeError as exc:
            last_error = exc
            try:
                REPORT.warn(f"JSON decode failed for {url}; retrying with curl")
                return json.loads(fetch_text_with_curl(url, headers=req_headers))
            except json.JSONDecodeError as curl_exc:
                last_error = curl_exc
                if attempt == 3:
                    break
            except Exception as curl_exc:
                last_error = curl_exc
                if attempt == 3:
                    break
    raise TestFailure(f"failed to parse JSON from {url}: {last_error}") from last_error


def fetch_url_and_final_location(url: str, *, headers: Optional[Dict[str, str]] = None) -> Tuple[str, str]:
    req_headers = {"User-Agent": USER_AGENT}
    if headers:
        req_headers.update(headers)
    last_error: Optional[Exception] = None
    for attempt in range(1, 4):
        request = urllib.request.Request(url, headers=req_headers)
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return "", response.geturl()
        except (urllib.error.URLError, TimeoutError, http.client.HTTPException, http.client.IncompleteRead) as exc:
            last_error = exc
            if attempt == 3:
                break
        except Exception as exc:
            last_error = exc
            if attempt == 3:
                break
    try:
        REPORT.warn(f"urllib final-url fetch failed for {url}; retrying with curl")
        return "", fetch_final_url_with_curl(url, headers=req_headers)
    except Exception as curl_exc:
        raise TestFailure(f"failed to fetch {url}: {last_error}; curl fallback failed: {curl_exc}") from curl_exc


def read_response_bytes(response) -> bytes:
    chunks: List[bytes] = []
    while True:
        try:
            chunk = response.read(64 * 1024)
        except http.client.IncompleteRead as exc:
            raise exc
        if not chunk:
            break
        chunks.append(chunk)
    return b"".join(chunks)


def extract_archive_bins_from_stream(stream, pattern: re.Pattern[str]) -> Dict[str, str]:
    bins: Dict[str, str] = {}
    with tarfile.open(fileobj=stream, mode="r|gz") as archive:
        for member in archive:
            if not (member.isfile() or member.issym() or member.islnk()):
                continue
            match = pattern.search(member.name)
            if not match:
                continue
            rel_path = match.group(1)
            bins[Path(rel_path).name] = rel_path
    if not bins:
        raise TestFailure("no binaries discovered in archive")
    return dict(sorted(bins.items()))


def extract_archive_bins(url: str, pattern: re.Pattern[str]) -> Dict[str, str]:
    last_error: Optional[Exception] = None
    for attempt in range(1, 4):
        request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
        try:
            with urllib.request.urlopen(request, timeout=120) as response:
                return extract_archive_bins_from_stream(response, pattern)
        except (urllib.error.URLError, TimeoutError, http.client.HTTPException, http.client.IncompleteRead) as exc:
            last_error = exc
            if attempt == 3:
                break
        except tarfile.TarError as exc:
            last_error = exc
            if attempt == 3:
                break
    raise TestFailure(f"failed to stream archive {url}: {last_error}") from last_error


def extract_archive_bins_from_file(path: Path, pattern: re.Pattern[str]) -> Dict[str, str]:
    try:
        with path.open("rb") as archive_file:
            return extract_archive_bins_from_stream(archive_file, pattern)
    except (OSError, tarfile.TarError) as exc:
        raise TestFailure(f"failed to read archive {path}: {exc}") from exc


def download_archive_with_curl(url: str, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    if shutil.which("curl") is None:
        raise TestFailure("curl is required for fallback archive download but was not found")
    REPORT.info(f"Downloading official archive to {destination.name} (progress below)")
    run_cmd_live(
        [
            "curl",
            "-L",
            "--fail",
            "--retry",
            "3",
            "--retry-all-errors",
            "--progress-bar",
            "--output",
            str(destination),
            url,
        ],
        timeout=3600,
        allow_nonzero=False,
    )


def fetch_text_with_curl(url: str, *, headers: Optional[Dict[str, str]] = None) -> str:
    if shutil.which("curl") is None:
        raise TestFailure("curl fallback requested but curl was not found")
    cmd = ["curl", "-L", "--fail", "--retry", "3", "--retry-all-errors", "--silent", "--show-error"]
    if headers:
        for key, value in headers.items():
            cmd.extend(["-H", f"{key}: {value}"])
    cmd.append(url)
    return run_cmd(cmd, timeout=300, allow_nonzero=False).output


def fetch_final_url_with_curl(url: str, *, headers: Optional[Dict[str, str]] = None) -> str:
    if shutil.which("curl") is None:
        raise TestFailure("curl fallback requested but curl was not found")
    cmd = [
        "curl",
        "-L",
        "--fail",
        "--retry",
        "3",
        "--retry-all-errors",
        "--silent",
        "--show-error",
        "--output",
        "/dev/null",
        "--write-out",
        "%{url_effective}",
    ]
    if headers:
        for key, value in headers.items():
            cmd.extend(["-H", f"{key}: {value}"])
    cmd.append(url)
    return run_cmd(cmd, timeout=300, allow_nonzero=False).output.strip()


def ensure_upstream_bins(plan: ToolPlan) -> Dict[str, str]:
    if plan.upstream_bins:
        return plan.upstream_bins

    cache_archive = plan.cache_archive
    if cache_archive.exists():
        REPORT.info(f"{plan.display_name}: scanning cached official archive {cache_archive.name}")
        plan.upstream_bins = extract_archive_bins_from_file(cache_archive, plan.bin_regex)
        return plan.upstream_bins

    audit_archive = plan.audit_archive
    if audit_archive.exists() and not FRESH_RUN:
        REPORT.info(f"{plan.display_name}: scanning saved audit archive {audit_archive.name}")
        plan.upstream_bins = extract_archive_bins_from_file(audit_archive, plan.bin_regex)
        return plan.upstream_bins

    if audit_archive.exists():
        audit_archive.unlink()
    REPORT.info(f"{plan.display_name}: downloading a fresh official archive for comparison")
    download_archive_with_curl(plan.download_url, audit_archive)
    plan.upstream_bins = extract_archive_bins_from_file(audit_archive, plan.bin_regex)
    return plan.upstream_bins


def resolve_prefix_version(spec: str, versions: Iterable[str]) -> str:
    normalized = spec.lstrip("v")
    versions_list = list(versions)
    if normalized in versions_list:
        return normalized
    prefix = normalized + "."
    for version in versions_list:
        if version.startswith(prefix):
            return version
    raise TestFailure(f"could not resolve version spec '{spec}' from upstream data")


def semver_key(version: str) -> Tuple[int, ...]:
    return tuple(int(part) for part in re.findall(r"\d+", version))


def build_tool_plans() -> Dict[str, ToolPlan]:
    REPORT.section("Upstream Resolution")
    resolvers = [
        ("node", "Node.js", resolve_node, os.environ.get("NODE_SPEC", "lts")),
        ("go", "Go", resolve_go, os.environ.get("GO_SPEC", "latest")),
        ("java", "Java", resolve_java, os.environ.get("JAVA_SPEC", "lts")),
        ("rust", "Rust", resolve_rust, os.environ.get("RUST_SPEC", "stable")),
        ("python", "Python", resolve_python, os.environ.get("PYTHON_SPEC", "latest")),
    ]
    plans: Dict[str, ToolPlan] = {}
    for tool_name, display_name, resolver, spec in resolvers:
        REPORT.info(f"{display_name}: starting upstream resolution for spec '{spec}'")
        try:
            plans[tool_name] = resolver(spec)
            REPORT.ok(
                f"{display_name}: resolved {spec} -> {plans[tool_name].resolved_version}"
            )
        except Exception as exc:
            raise TestFailure(f"{display_name} upstream resolution failed: {exc}") from exc
    return plans


def resolve_node(spec: str) -> ToolPlan:
    REPORT.info("Node.js: fetching official version index")
    table = fetch_text("https://nodejs.org/dist/index.tab")
    reader = csv.DictReader(table.splitlines(), delimiter="\t")
    normalized = []
    for row in reader:
        if not row or not row.get("version"):
            continue
        lts = (row.get("lts") or "").strip() or None
        if lts == "-":
            lts = None
        normalized.append(
            {
                "version": row["version"].lstrip("v"),
                "lts": lts,
            }
        )
    if spec == "latest":
        resolved = normalized[0]["version"]
    elif spec == "lts":
        resolved = next(item["version"] for item in normalized if item["lts"])
    elif spec.startswith("lts-"):
        codename = spec[4:].lower()
        resolved = next(
            item["version"]
            for item in normalized
            if item["lts"] and str(item["lts"]).lower() == codename
        )
    else:
        resolved = resolve_prefix_version(spec, [item["version"] for item in normalized])

    arch = "arm64" if os.uname().machine in {"arm64", "aarch64"} else "x64"
    download_url = f"https://nodejs.org/dist/v{resolved}/node-v{resolved}-darwin-{arch}.tar.gz"
    return ToolPlan(
        name="node",
        display_name="Node.js",
        requested_spec=spec,
        resolved_version=resolved,
        download_url=download_url,
        upstream_bins={},
        install_spec=f"node@{resolved}",
        bin_regex=NODE_BIN_RE,
        meta={"releases": normalized},
    )


def resolve_go(spec: str) -> ToolPlan:
    REPORT.info("Go: fetching official release metadata")
    releases = fetch_json("https://go.dev/dl/?mode=json")
    stable = [item for item in releases if item.get("stable")]
    versions = [item["version"].removeprefix("go") for item in stable]
    if spec == "latest":
        resolved = versions[0]
    else:
        resolved = resolve_prefix_version(spec, versions)

    arch = "arm64" if os.uname().machine in {"arm64", "aarch64"} else "amd64"
    download_url = f"https://go.dev/dl/go{resolved}.darwin-{arch}.tar.gz"
    return ToolPlan(
        name="go",
        display_name="Go",
        requested_spec=spec,
        resolved_version=resolved,
        download_url=download_url,
        upstream_bins={},
        install_spec=f"go@{resolved}",
        bin_regex=GO_BIN_RE,
    )


def resolve_java(spec: str) -> ToolPlan:
    REPORT.info("Java: fetching Adoptium release metadata")
    releases = fetch_json("https://api.adoptium.net/v3/info/available_releases")
    available = sorted((int(v) for v in releases["available_releases"]), reverse=True)
    lts = sorted((int(v) for v in releases["available_lts_releases"]), reverse=True)
    if spec == "latest":
        resolved = str(available[0])
    elif spec == "lts":
        resolved = str(lts[0])
    else:
        resolved = str(int(spec))

    arch = "aarch64" if os.uname().machine in {"arm64", "aarch64"} else "x64"
    asset_url = (
        f"https://api.adoptium.net/v3/assets/latest/{resolved}/hotspot"
        f"?architecture={arch}&image_type=jdk&os=mac&vendor=eclipse"
    )
    assets = fetch_json(asset_url)
    if not assets:
        raise TestFailure(f"Adoptium returned no macOS JDK asset for Java {resolved}")
    download_url = assets[0]["binary"]["package"]["link"]
    return ToolPlan(
        name="java",
        display_name="Java",
        requested_spec=spec,
        resolved_version=resolved,
        download_url=download_url,
        upstream_bins={},
        install_spec=f"java@{resolved}",
        bin_regex=JAVA_BIN_RE,
    )


def resolve_rust(spec: str) -> ToolPlan:
    REPORT.info("Rust: fetching stable channel manifest")
    manifest = fetch_text("https://static.rust-lang.org/dist/channel-rust-stable.toml")
    match = re.search(r"(?ms)^\[pkg\.rust\].*?^version = \"([^\"]+)\"", manifest)
    if not match:
        raise TestFailure("could not parse stable Rust version from official manifest")
    stable_version = match.group(1).split()[0]
    resolved = stable_version if spec in {"latest", "stable"} else spec
    target = "aarch64-apple-darwin" if os.uname().machine in {"arm64", "aarch64"} else "x86_64-apple-darwin"
    download_url = f"https://static.rust-lang.org/dist/rust-{resolved}-{target}.tar.gz"
    return ToolPlan(
        name="rust",
        display_name="Rust",
        requested_spec=spec,
        resolved_version=resolved,
        download_url=download_url,
        upstream_bins={},
        install_spec=f"rust@{resolved}",
        bin_regex=RUST_BIN_RE,
    )


def resolve_python(spec: str) -> ToolPlan:
    REPORT.info("Python: fetching latest release tag")
    _, final_url = fetch_url_and_final_location(
        "https://github.com/astral-sh/python-build-standalone/releases/latest"
    )
    tag = final_url.rstrip("/").split("/")[-1]
    if not tag or tag == "latest":
        raise TestFailure(
            f"could not determine python-build-standalone latest release tag from {final_url}"
        )

    REPORT.info(f"Python: fetching SHA256SUMS for release {tag}")
    sha256sums = fetch_text(
        f"https://github.com/astral-sh/python-build-standalone/releases/download/{tag}/SHA256SUMS"
    )
    arch_fragment = "aarch64-apple-darwin" if os.uname().machine in {"arm64", "aarch64"} else "x86_64-apple-darwin"
    asset_names = []
    for line in sha256sums.splitlines():
        parts = line.strip().split(None, 1)
        if len(parts) != 2:
            continue
        name = parts[1].strip()
        if (
            name.endswith("install_only.tar.gz")
            and "freethreaded" not in name
            and "stripped" not in name
            and arch_fragment in name
            and name.startswith("cpython-")
        ):
            asset_names.append(name)

    versions = sorted(
        {
            re.match(r"cpython-([0-9]+\.[0-9]+\.[0-9]+)\+", asset_name).group(1)
            for asset_name in asset_names
            if re.match(r"cpython-([0-9]+\.[0-9]+\.[0-9]+)\+", asset_name)
        },
        key=semver_key,
        reverse=True,
    )
    if not versions:
        raise TestFailure(
            f"no matching Python install_only assets found in SHA256SUMS for {tag} ({arch_fragment})"
        )
    if spec in {"latest", "stable", "bugfix"}:
        resolved = versions[0]
    else:
        resolved = resolve_prefix_version(spec, versions)

    matching_name = (
        f"cpython-{resolved}+{tag}-{arch_fragment}-install_only.tar.gz"
    )
    if not matching_name:
        raise TestFailure(f"python-build-standalone has no matching asset for {resolved}")
    if matching_name not in asset_names:
        raise TestFailure(
            f"python-build-standalone has no standard install_only asset for {resolved}"
        )
    download_url = (
        f"https://github.com/astral-sh/python-build-standalone/releases/download/{tag}/{matching_name}"
    )
    return ToolPlan(
        name="python",
        display_name="Python",
        requested_spec=spec,
        resolved_version=resolved,
        download_url=download_url,
        upstream_bins={},
        install_spec=f"python@{resolved}",
        bin_regex=PYTHON_BIN_RE,
    )


def choose_alt_node_version(plan: ToolPlan) -> Optional[str]:
    releases = plan.meta.get("releases")
    if not isinstance(releases, list):
        return None
    major = plan.resolved_version.split(".", 1)[0]
    for item in releases:
        version = str(item["version"])
        if version != plan.resolved_version and version.startswith(f"{major}."):
            return version
    for item in releases:
        version = str(item["version"])
        if version != plan.resolved_version:
            return version
    return None


def validate_tool(plan: ToolPlan) -> None:
    REPORT.section(f"{plan.display_name} {plan.resolved_version}")
    REPORT.info(f"Official archive: {plan.download_url}")

    REPORT.info(f"{plan.display_name}: running fresh vex install (download progress below)")
    run_cmd_live([str(VEX_BIN), "install", plan.install_spec], timeout=1800)
    upstream_bins = ensure_upstream_bins(plan)
    REPORT.ok(
        f"{plan.display_name}: discovered {len(upstream_bins)} binaries from the official archive"
    )
    run_cmd([str(VEX_BIN), "use", plan.install_spec], timeout=300)
    current_output = run_cmd([str(VEX_BIN), "current"], timeout=60).output
    REPORT.expect(
        plan.name in current_output and plan.resolved_version in current_output,
        f"vex current includes {plan.name}@{plan.resolved_version}",
        f"vex current missing {plan.name}@{plan.resolved_version}",
    )

    local_bins = collect_local_toolchain_bins(plan)
    compare_bin_maps(
        f"{plan.display_name} toolchain contents",
        expected=upstream_bins,
        actual=local_bins,
    )

    current_link = TEST_HOME / ".vex" / "current" / plan.name
    REPORT.expect(current_link.is_symlink(), f"{current_link} exists", f"missing current symlink {current_link}")
    if current_link.is_symlink():
        target = current_link.resolve()
        REPORT.expect(
            target == plan.toolchain_root.resolve(),
            f"current/{plan.name} points to {plan.resolved_version}",
            f"current/{plan.name} points to {target}",
        )

    linked_bins = collect_linked_bins(plan)
    compare_bin_maps(
        f"{plan.display_name} ~/.vex/bin symlinks",
        expected=upstream_bins,
        actual=linked_bins,
    )

    for bin_name, rel_path in sorted(upstream_bins.items()):
        link_path = TEST_HOME / ".vex" / "bin" / bin_name
        expected_target = plan.toolchain_root / rel_path
        REPORT.expect(link_path.is_symlink(), f"{bin_name} symlink exists", f"{bin_name} symlink missing")
        if not link_path.is_symlink():
            continue
        try:
            actual_target = link_path.resolve()
        except FileNotFoundError:
            REPORT.fail(f"{bin_name} symlink is broken")
            continue
        REPORT.expect(
            actual_target == expected_target.resolve(),
            f"{bin_name} points to {rel_path}",
            f"{bin_name} points to {actual_target}, expected {expected_target}",
        )
        resolved_path = shutil.which(bin_name, path=env_for_subprocess()["PATH"])
        REPORT.expect(
            resolved_path == str(link_path),
            f"command -v {bin_name} resolves to ~/.vex/bin/{bin_name}",
            f"command -v {bin_name} resolved to {resolved_path}",
        )

    probe_binaries(plan)


def compare_bin_maps(title: str, *, expected: Dict[str, str], actual: Dict[str, str]) -> None:
    missing = sorted(set(expected) - set(actual))
    extra = sorted(set(actual) - set(expected))
    mismatched = sorted(name for name in set(expected) & set(actual) if expected[name] != actual[name])

    if not missing and not extra and not mismatched:
        REPORT.ok(f"{title} match upstream exactly ({len(expected)} bins)")
        return

    if missing:
        REPORT.fail(f"{title} missing: {', '.join(missing)}")
    if extra:
        REPORT.fail(f"{title} unexpected extra bins: {', '.join(extra)}")
    for name in mismatched:
        REPORT.fail(f"{title} path mismatch for {name}: local={actual[name]} upstream={expected[name]}")


def collect_local_toolchain_bins(plan: ToolPlan) -> Dict[str, str]:
    bins: Dict[str, str] = {}
    if not plan.toolchain_root.exists():
        raise TestFailure(f"missing toolchain root {plan.toolchain_root}")

    expected_dirs = sorted({Path(rel_path).parent for rel_path in plan.upstream_bins.values()})
    for rel_dir in expected_dirs:
        abs_dir = plan.toolchain_root / rel_dir
        if not abs_dir.is_dir():
            continue
        for entry in abs_dir.iterdir():
            if entry.is_dir():
                continue
            if not entry.exists() and not entry.is_symlink():
                continue
            bins[entry.name] = entry.relative_to(plan.toolchain_root).as_posix()
    return dict(sorted(bins.items()))


def collect_linked_bins(plan: ToolPlan) -> Dict[str, str]:
    bin_dir = TEST_HOME / ".vex" / "bin"
    bins: Dict[str, str] = {}
    if not bin_dir.exists():
        return bins
    for entry in bin_dir.iterdir():
        if not entry.is_symlink():
            continue
        try:
            target = entry.readlink()
        except OSError:
            continue
        if not target.is_absolute():
            target = Path(os.path.normpath(str(entry.parent / target)))
        try:
            rel = target.relative_to(plan.toolchain_root).as_posix()
        except ValueError:
            continue
        bins[entry.name] = rel
    return dict(sorted(bins.items()))


def probe_binaries(plan: ToolPlan) -> None:
    version_major = plan.resolved_version.split(".")[0]
    version_minor = ".".join(plan.resolved_version.split(".")[:2])
    for bin_name in sorted(plan.upstream_bins):
        path = TEST_HOME / ".vex" / "bin" / bin_name
        if plan.name == "rust" and bin_name in RUST_WRAPPER_BINS:
            validate_wrapper_script(path)
            continue
        if plan.name == "java" and bin_name in JAVA_STRUCTURAL_BINS:
            validate_executable_artifact(path)
            continue
        probes = binary_probes(plan, bin_name, version_major, version_minor)
        result = run_probe_sequence(path, probes)
        if result is None:
            REPORT.fail(f"{bin_name} did not respond correctly to any probe")
        else:
            REPORT.ok(f"{bin_name} probe passed via {' '.join(result.cmd[1:])}")


def binary_probes(plan: ToolPlan, bin_name: str, version_major: str, version_minor: str) -> List[Probe]:
    if plan.name == "node":
        if bin_name == "node":
            return [Probe(["--version"], [rf"v{re.escape(version_major)}\."], False), Probe(["--help"], [USAGE_RE], True)]
        if bin_name in {"npm", "npx", "corepack"}:
            return [Probe(["--version"], [r"\d+\.\d+\.\d+"], False), Probe(["--help"], [bin_name, USAGE_RE], True)]

    if plan.name == "go":
        if bin_name == "go":
            return [Probe(["version"], [rf"go version go{re.escape(plan.resolved_version)}"], False), Probe(["help"], [r"Go is a tool"], True)]
        return [Probe(["-h"], [r"usage"], True), Probe(["--help"], [r"usage"], True)]

    if plan.name == "python":
        if bin_name.startswith("python") and not bin_name.endswith("-config"):
            return [
                Probe(["--version"], [rf"Python {re.escape(version_minor)}"], False),
                Probe(["-V"], [rf"Python {re.escape(version_minor)}"], False),
                Probe(["--help"], [USAGE_RE], True),
            ]
        if bin_name.startswith("pip"):
            return [
                Probe(["--version"], [r"pip ", rf"python {re.escape(version_minor)}"], False),
                Probe(["--help"], [USAGE_RE], True),
            ]
        if bin_name.startswith("2to3"):
            return [Probe(["--help"], [USAGE_RE], True), Probe(["-h"], [USAGE_RE], True)]
        if bin_name.startswith("pydoc"):
            return [Probe(["-h"], [r"pydoc"], True), Probe(["--help"], [r"pydoc"], True)]
        if bin_name.startswith("idle"):
            return [Probe(["-h"], [USAGE_RE], True), Probe(["--help"], [USAGE_RE], True)]
        if bin_name.endswith("-config"):
            return [Probe(["--help"], [USAGE_RE], True), Probe(["--prefix"], [r"/"], False)]

    if plan.name == "rust":
        if bin_name in {"rustc", "cargo"}:
            return [
                Probe(["--version"], [rf"{bin_name} {re.escape(plan.resolved_version)}"], False),
                Probe(["-V"], [rf"{bin_name} {re.escape(plan.resolved_version)}"], False),
            ]
        if bin_name == "rustdoc":
            return [Probe(["--version"], [r"rustdoc", re.escape(plan.resolved_version)], False), Probe(["--help"], [USAGE_RE], True)]
        if bin_name in {"rustfmt", "cargo-fmt"}:
            return [Probe(["--version"], [r"rustfmt"], False), Probe(["--help"], [r"Format"], True)]
        if bin_name in {"clippy-driver", "cargo-clippy"}:
            return [Probe(["--version"], [r"clippy"], True), Probe(["--help"], [r"clippy|Checks a package"], True)]
        if bin_name == "rust-analyzer":
            return [Probe(["--version"], [r"rust-analyzer"], False), Probe(["--help"], [USAGE_RE], True)]

    if plan.name == "java":
        if bin_name in {"java", "javac", "javap", "jdb", "jps", "jstack", "jstat"}:
            return [Probe(["-version"], [re.escape(version_major)], True), Probe(["-help"], [USAGE_RE], True)]
        if bin_name in {"jar", "javadoc", "jshell", "jdeps", "jfr", "jlink", "jmod", "jpackage", "jwebserver", "jcmd", "jdeprscan", "jimage"}:
            return [Probe(["--version"], [re.escape(version_major)], True), Probe(["--help"], [USAGE_RE], True)]
        if bin_name == "jhsdb":
            return [Probe(["--help"], [r"clhsdb|hsdb|jstack"], True), Probe(["-help"], [r"clhsdb|hsdb|jstack"], True)]
        return [
            Probe(["--help"], [USAGE_RE], True),
            Probe(["-help"], [USAGE_RE], True),
            Probe(["-?"], [USAGE_RE], True),
            Probe(["-version"], [re.escape(version_major)], True),
        ]

    return [
        Probe(["--version"], [bin_name], True),
        Probe(["-V"], [bin_name], True),
        Probe(["--help"], [USAGE_RE, bin_name], True),
        Probe(["-h"], [USAGE_RE, bin_name], True),
    ]


def run_probe_sequence(path: Path, probes: Sequence[Probe]) -> Optional[CommandResult]:
    for probe in probes:
        try:
            result = run_cmd([str(path), *probe.args], timeout=12, allow_nonzero=True)
        except subprocess.TimeoutExpired:
            continue
        output = result.output.strip()
        lowered = output.lower()
        if result.returncode != 0 and not probe.allow_nonzero:
            continue
        if any(re.search(pattern, lowered, re.IGNORECASE) for pattern in BAD_EXEC_PATTERNS):
            continue
        if not output:
            continue
        if probe.expect and not all(re.search(pattern, output, re.IGNORECASE) for pattern in probe.expect):
            continue
        return result
    return None


def validate_wrapper_script(path: Path) -> None:
    REPORT.expect(path.exists(), f"{path.name} wrapper exists", f"{path.name} wrapper missing")
    if not path.exists():
        return
    mode = path.stat().st_mode
    REPORT.expect(mode & stat.S_IXUSR != 0, f"{path.name} is executable", f"{path.name} is not executable")
    lines = path.read_text(errors="ignore").splitlines() if path.is_file() else []
    first_line = lines[0] if lines else ""
    REPORT.expect(first_line.startswith("#!"), f"{path.name} has a shebang", f"{path.name} is missing a shebang")
    REPORT.ok(f"{path.name} validated structurally (wrapper script)")


def validate_executable_artifact(path: Path) -> None:
    REPORT.expect(path.exists(), f"{path.name} artifact exists", f"{path.name} artifact missing")
    if not path.exists():
        return
    mode = path.stat().st_mode
    REPORT.expect(mode & stat.S_IXUSR != 0, f"{path.name} is executable", f"{path.name} is not executable")
    REPORT.ok(f"{path.name} validated structurally (non-interactive probe skipped)")


def validate_python_venv(plan: ToolPlan) -> None:
    REPORT.section("Python venv workflow")
    project = PROJECT_ROOT
    project.mkdir(parents=True, exist_ok=True)
    write_text(project / ".tool-versions", f"python {plan.resolved_version}\n")

    run_cmd([str(VEX_BIN), "use", plan.install_spec], timeout=120)

    freeze_missing = run_cmd([str(VEX_BIN), "python", "freeze"], cwd=project, allow_nonzero=True)
    REPORT.expect(
        "No .venv found" in freeze_missing.output,
        "python freeze fails cleanly without .venv",
        f"unexpected freeze-without-venv output: {freeze_missing.output.strip()}",
    )

    sync_missing = run_cmd([str(VEX_BIN), "python", "sync"], cwd=project, allow_nonzero=True)
    REPORT.expect(
        "No requirements.lock found" in sync_missing.output,
        "python sync fails cleanly without requirements.lock",
        f"unexpected sync-without-lock output: {sync_missing.output.strip()}",
    )

    run_cmd([str(VEX_BIN), "python", "init"], cwd=project, timeout=300)
    REPORT.expect((project / ".venv").is_dir(), ".venv directory created", ".venv directory missing")
    REPORT.expect((project / ".venv" / "bin" / "python").exists(), ".venv/bin/python exists", ".venv/bin/python missing")
    REPORT.expect((project / ".venv" / "bin" / "pip").exists(), ".venv/bin/pip exists", ".venv/bin/pip missing")
    REPORT.expect((project / ".venv" / "bin" / "activate").exists(), ".venv activate script exists", ".venv activate script missing")

    venv_python = run_cmd([str(project / ".venv" / "bin" / "python"), "--version"], timeout=30)
    REPORT.expect(
        f"Python {'.'.join(plan.resolved_version.split('.')[:2])}" in venv_python.output,
        f"venv python reports {plan.resolved_version}",
        f"venv python version mismatch: {venv_python.output.strip()}",
    )

    run_cmd([str(VEX_BIN), "python", "freeze"], cwd=project, timeout=120)
    lock_path = project / "requirements.lock"
    REPORT.expect(lock_path.exists(), "requirements.lock created", "requirements.lock missing after freeze")

    shutil.rmtree(project / ".venv")
    run_cmd([str(VEX_BIN), "python", "sync"], cwd=project, timeout=300)
    REPORT.expect((project / ".venv").is_dir(), ".venv restored by sync", ".venv missing after sync")
    restored_pip = run_cmd([str(project / ".venv" / "bin" / "pip"), "--version"], timeout=30)
    REPORT.expect("pip " in restored_pip.output, "restored venv pip is runnable", f"restored pip failed: {restored_pip.output.strip()}")


def validate_project_behavior(plans: Dict[str, ToolPlan], alt_node_version: Optional[str]) -> None:
    REPORT.section("Project directory detection")
    workspace = TMP_ROOT / "workspace"
    project = PROJECT_ROOT
    workspace.mkdir(parents=True, exist_ok=True)
    project.mkdir(parents=True, exist_ok=True)

    for tool_name in ["go", "java", "rust", "python"]:
        run_cmd([str(VEX_BIN), "global", plans[tool_name].install_spec], timeout=120)
    run_cmd([str(VEX_BIN), "global", plans["node"].install_spec], timeout=120)

    if alt_node_version:
        REPORT.info(f"Project test: installing alternate global Node {alt_node_version} (progress below)")
        run_cmd_live([str(VEX_BIN), "install", f"node@{alt_node_version}"], timeout=1800)
        run_cmd([str(VEX_BIN), "global", f"node@{alt_node_version}"], timeout=120)
        run_cmd([str(VEX_BIN), "use", f"node@{alt_node_version}"], timeout=120)
        REPORT.ok(f"alternate global Node installed for hook test: {alt_node_version}")
    else:
        REPORT.warn("could not find an alternate Node version; hook version-swap check will be weaker")

    tool_versions = "\n".join(
        f"{name} {plans[name].resolved_version}" for name in ["node", "go", "java", "rust", "python"]
    ) + "\n"
    write_text(project / ".tool-versions", tool_versions)

    global_current = run_cmd([str(VEX_BIN), "current"], cwd=workspace, timeout=60).output
    REPORT.expect("Global default" in global_current, "outside project shows Global default", f"unexpected global current output: {global_current.strip()}")

    run_cmd([str(VEX_BIN), "use", "--auto"], cwd=project, timeout=120)
    project_current = run_cmd([str(VEX_BIN), "current"], cwd=project, timeout=60).output
    REPORT.expect("Project override" in project_current, "inside project shows Project override", f"unexpected project current output: {project_current.strip()}")
    for name in ["node", "go", "java", "rust", "python"]:
        REPORT.expect(
            name in project_current and plans[name].resolved_version in project_current,
            f"project current includes {name}@{plans[name].resolved_version}",
            f"project current missing {name}@{plans[name].resolved_version}",
        )

    zsh_script = textwrap.dedent(
        f"""
        export HOME={shell_quote(str(TEST_HOME))}
        export PATH={shell_quote(str(VEX_BIN.parent))}:$PATH
        eval "$({shell_quote(str(VEX_BIN))} env zsh)"
        cd {shell_quote(str(workspace))}
        echo "OUTSIDE_NODE=$(node --version)"
        cd {shell_quote(str(project))}
        echo "INSIDE_NODE=$(node --version)"
        echo "VIRTUAL_ENV=${{VIRTUAL_ENV:-}}"
        {shell_quote(str(VEX_BIN))} current
        """
    ).strip()
    hook_result = run_cmd(["zsh", "-lc", zsh_script], timeout=120, allow_nonzero=False)
    outside_match = re.search(r"OUTSIDE_NODE=(.+)", hook_result.output)
    inside_match = re.search(r"INSIDE_NODE=(.+)", hook_result.output)
    vex_match = re.search(r"VIRTUAL_ENV=(.+)", hook_result.output)

    expected_inside_node = f"v{plans['node'].resolved_version}"
    REPORT.expect(
        inside_match is not None and inside_match.group(1).strip() == expected_inside_node,
        f"cd hook activates project Node {plans['node'].resolved_version}",
        f"inside-project node version mismatch: {hook_result.output.strip()}",
    )
    if alt_node_version and outside_match:
        REPORT.expect(
            outside_match.group(1).strip() == f"v{alt_node_version}",
            f"outside project Node stays on global {alt_node_version}",
            f"outside-project node version mismatch: {hook_result.output.strip()}",
        )
    expected_venv = (project / ".venv").resolve()
    actual_venv = Path(vex_match.group(1).strip()).resolve() if vex_match is not None and vex_match.group(1).strip() else None
    REPORT.expect(
        actual_venv == expected_venv,
        "cd hook auto-activates project .venv",
        f"VIRTUAL_ENV not auto-activated: {hook_result.output.strip()}",
    )
    REPORT.expect(
        "Project override" in hook_result.output,
        "hook-driven vex current shows Project override",
        f"hook-driven vex current output missing Project override: {hook_result.output.strip()}",
    )


def validate_doctor() -> None:
    REPORT.section("Doctor")
    doctor = run_cmd([str(VEX_BIN), "doctor"], timeout=180, allow_nonzero=True)
    output = doctor.output
    REPORT.expect("Checking vex directory" in output, "doctor checks vex directory", f"doctor output missing directory check: {output.strip()}")
    REPORT.expect("Checking symlinks integrity" in output, "doctor checks symlinks", f"doctor output missing symlink check: {output.strip()}")
    REPORT.expect("Checking binary runnability" in output, "doctor checks binary runnability", f"doctor output missing runnability check: {output.strip()}")
    REPORT.expect("Error:" not in output, "doctor has no fatal error banner", f"doctor reported an error: {output.strip()}")


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def shell_quote(value: str) -> str:
    return "'" + value.replace("'", "'\"'\"'") + "'"


if __name__ == "__main__":
    raise SystemExit(main())
