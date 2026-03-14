#!/usr/bin/env python3
"""
Strict macOS validation for vex binaries.

What it checks:
- Downloads the latest official vex macOS release binary fresh from GitHub, or
  uses a specified local build.
- Verifies top-level help, subcommand help coverage, shell hook output, and init flows.
- Resolves official macOS archives for node/go/java/rust/python from upstream.
- Installs and activates each tool with the vex binary under test.
- Verifies toolchain layout, current symlink, ~/.vex/bin symlinks, command resolution,
  and runnable probes.
- Exercises Python venv lifecycle and project-directory auto-switch / auto-activation.

Notes:
- By default this runs in an isolated HOME if VEX_TEST_HOME is set.
- The strict path can test either a fresh official release or a local build.
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


USER_AGENT = "vex-macos-strict-test/1.0"
REPO_ROOT = Path(__file__).resolve().parents[1]
REAL_HOME = Path.home()
TEST_HOME = Path(os.environ.get("VEX_TEST_HOME", str(REAL_HOME))).expanduser().resolve()
LOCAL_VEX_BIN_STR = shutil.which("vex")
LOCAL_VEX_BIN = Path(LOCAL_VEX_BIN_STR).expanduser().resolve() if LOCAL_VEX_BIN_STR else None
VEX_BIN: Optional[Path] = None
STRICT_USE_LOCAL_BUILD = os.environ.get("VEX_STRICT_USE_LOCAL_BUILD", "").lower() in {"1", "true", "yes"}
STRICT_LOCAL_BUILD_BIN = Path(
    os.environ.get("VEX_STRICT_VEX_BIN", str(REPO_ROOT / "target" / "debug" / "vex"))
).expanduser()
VEX_RELEASE_API = os.environ.get(
    "VEX_RELEASE_API",
    "https://api.github.com/repos/imnotnoahhh/vex/releases/latest",
)
FRESH_RUN = os.environ.get("VEX_TEST_REUSE", "").lower() not in {"1", "true", "yes"}
STRICT_TMP_ROOT = os.environ.get("VEX_STRICT_TMP_ROOT", "").strip()
if STRICT_TMP_ROOT:
    TMP_ROOT = Path(STRICT_TMP_ROOT).expanduser().resolve()
    if FRESH_RUN and TMP_ROOT.exists():
        shutil.rmtree(TMP_ROOT)
    TMP_ROOT.mkdir(parents=True, exist_ok=True)
else:
    TMP_ROOT = Path(tempfile.mkdtemp(prefix="vex-macos-strict-"))
PROJECT_ROOT = TMP_ROOT / "project"
LOG_ROOT = TMP_ROOT / "logs"
VEX_RELEASE_ROOT = TMP_ROOT / "vex-release"
LOG_ROOT.mkdir(parents=True, exist_ok=True)


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
JAVA_FALLBACK_LTS_CANDIDATES = [25, 21, 17, 11, 8]

EXPECTED_TOP_LEVEL_COMMANDS = {
    "init": "Initialize vex directory structure",
    "install": "Install a tool version (or all from .tool-versions)",
    "use": "Switch to a different version",
    "list": "List installed versions",
    "list-remote": "List available remote versions",
    "current": "Show current active versions",
    "uninstall": "Uninstall a version",
    "env": "Output shell hook for auto-switching",
    "local": "Pin a tool version in the current directory (.tool-versions)",
    "global": "Pin a tool version globally (~/.vex/tool-versions)",
    "upgrade": "Upgrade a tool to the latest version",
    "outdated": "Show which managed tools are behind the latest available version",
    "prune": "Remove unused cache files, stale locks, and unreferenced toolchains",
    "alias": "Show available aliases for a tool",
    "exec": "Run a command inside the resolved vex-managed environment without switching global state",
    "run": "Run a named task from .vex.toml inside the resolved vex-managed environment",
    "doctor": "Check vex installation health",
    "self-update": "Update vex itself to the latest release",
    "python": "Python virtual environment management",
    "help": "Print this message or the help of the given subcommand(s)",
}

COMMAND_HELP_CHECKS = {
    "init": ["Usage: vex init", "--shell", "--dry-run"],
    "install": ["Usage: vex install", "--no-switch", "--force"],
    "use": ["Usage: vex use", "--auto"],
    "list": ["Usage: vex list", "<TOOL>"],
    "list-remote": ["Usage: vex list-remote", "--filter", "--no-cache"],
    "current": ["Usage: vex current"],
    "uninstall": ["Usage: vex uninstall", "<SPEC>"],
    "env": ["Usage: vex env", "<SHELL>"],
    "local": ["Usage: vex local", "<SPEC>"],
    "global": ["Usage: vex global", "<SPEC>"],
    "upgrade": ["Usage: vex upgrade", "[TOOL]", "--all"],
    "outdated": ["Usage: vex outdated", "[TOOL]"],
    "prune": ["Usage: vex prune", "--dry-run"],
    "alias": ["Usage: vex alias", "<TOOL>"],
    "exec": ["Usage: vex exec", "--", "<COMMAND>..."],
    "run": ["Usage: vex run", "<TASK>"],
    "doctor": ["Usage: vex doctor"],
    "self-update": ["Usage: vex self-update"],
    "python": ["Usage: vex python <SUBCMD>", "init", "freeze", "sync"],
}

ENV_HOOK_CHECKS = {
    "zsh": ["# vex shell integration", "add-zsh-hook chpwd", "__vex_use_if_found", "__vex_activate_venv"],
    "bash": ["# vex shell integration", "PROMPT_COMMAND", "__vex_use_if_found", "__vex_activate_venv"],
    "fish": ["# vex shell integration", "function __vex_use_if_found", "on-variable PWD", "__vex_activate_venv"],
    "nu": ["# vex shell integration", "def --env __vex_use_if_found", "pre_prompt", "__vex_activate_venv"],
}


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
class VexReleasePlan:
    source_kind: str
    source_label: str
    tag_name: str
    version: str
    asset_name: str
    download_url: str
    binary_path: Path


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
        if VEX_BIN is not None:
            print(f"Tested vex : {VEX_BIN}")
        print(f"Test HOME : {TEST_HOME}")
        print(f"Artifacts  : {TMP_ROOT}")
        return 1 if self.failed else 0


REPORT = Reporter()


def main() -> int:
    try:
        global VEX_BIN
        verify_prereqs()
        print_banner()

        REPORT.section("Environment")
        REPORT.expect(
            sys.platform == "darwin",
            "running on macOS",
            f"this script targets macOS but saw {sys.platform}",
        )
        if LOCAL_VEX_BIN:
            REPORT.ok(f"local vex binary discovered at {LOCAL_VEX_BIN}")
        else:
            REPORT.warn("no local vex binary found in PATH; strict mode will rely entirely on the downloaded release")
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

        vex_release = resolve_vex_under_test()
        VEX_BIN = vex_release.binary_path
        if vex_release.source_kind == "release":
            REPORT.ok(f"fresh vex release downloaded to {VEX_BIN}")
            REPORT.ok(f"strict mode is using release asset {vex_release.asset_name}")
            REPORT.expect(
                LOCAL_VEX_BIN is None or VEX_BIN.resolve() != LOCAL_VEX_BIN.resolve(),
                "strict mode uses a freshly downloaded vex binary",
                "strict mode unexpectedly resolved back to the local vex binary",
            )
        else:
            REPORT.ok(f"strict mode is using local build {VEX_BIN}")
            REPORT.ok(vex_release.source_label)
        vex_version = run_cmd([str(VEX_BIN), "--version"]).output.strip()
        REPORT.expect(
            vex_version == f"vex {vex_release.version}",
            f"vex under test reports version: {vex_version}",
            f"vex under test version mismatch: expected vex {vex_release.version}, saw {vex_version}",
        )

        validate_cli_help_surface(vex_release)
        validate_init_and_shells(vex_release)

        plans = build_tool_plans()
        alt_versions = choose_alt_versions(plans)

        for tool_name in ["node", "go", "java", "rust", "python"]:
            validate_tool(plans[tool_name])

        validate_python_venv(plans["python"])
        validate_manual_multiversion_switching(plans, alt_versions)
        validate_project_behavior(plans, alt_versions)
        validate_doctor()
        return REPORT.summary()
    except KeyboardInterrupt:
        print("\nInterrupted.")
        return 130
    except Exception as exc:
        REPORT.fail(str(exc))
        return REPORT.summary()


def verify_prereqs() -> None:
    if shutil.which("zsh") is None:
        raise TestFailure("zsh is required for the auto-switch test")
    if shutil.which("curl") is None:
        raise TestFailure("curl is required for strict release download and archive comparison")


def print_banner() -> None:
    print()
    print("============================================================")
    print("vex macOS strict validation")
    print("Release or local-build binary + init/help/env + upstream diff + venv + cd hook")
    print("============================================================")


def reset_test_home() -> None:
    vex_home = TEST_HOME / ".vex"
    targets = [
        vex_home,
        TEST_HOME / ".zshrc",
        TEST_HOME / ".bashrc",
        TEST_HOME / ".bash_profile",
        TEST_HOME / ".config" / "fish",
        TEST_HOME / ".config" / "nushell",
    ]
    for target in targets:
        if target.is_dir():
            shutil.rmtree(target, ignore_errors=True)
        elif target.exists() or target.is_symlink():
            try:
                target.unlink()
            except FileNotFoundError:
                pass


def env_for_subprocess(
    *,
    home: Optional[Path] = None,
    vex_bin: Optional[Path] = None,
) -> Dict[str, str]:
    env = os.environ.copy()
    active_home = home or TEST_HOME
    active_vex = vex_bin or VEX_BIN
    env["HOME"] = str(active_home)
    path_parts = [str(active_home / ".vex" / "bin")]
    if active_vex is not None:
        path_parts.append(str(active_vex.parent))
    current_path = env.get("PATH", "")
    env["PATH"] = ":".join([part for part in [*path_parts, current_path] if part])
    env["CARGO_HOME"] = str(active_home / ".vex" / "cargo")
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


def download_archive_with_curl(url: str, destination: Path, *, label: str = "official archive") -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    if shutil.which("curl") is None:
        raise TestFailure("curl is required for fallback archive download but was not found")
    REPORT.info(f"Downloading {label} to {destination.name} (progress below)")
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


def strip_v_prefix(version: str) -> str:
    return version[1:] if version.startswith("v") else version


def vex_release_arch_suffix() -> str:
    return "aarch64-apple-darwin" if os.uname().machine in {"arm64", "aarch64"} else "x86_64-apple-darwin"


def resolve_local_build_vex() -> VexReleasePlan:
    binary_path = STRICT_LOCAL_BUILD_BIN.resolve()
    if not binary_path.exists():
        raise TestFailure(f"local vex build does not exist at {binary_path}")
    if not binary_path.is_file():
        raise TestFailure(f"local vex build path is not a file: {binary_path}")
    if not os.access(binary_path, os.X_OK):
        raise TestFailure(f"local vex build is not executable: {binary_path}")

    REPORT.info(f"vex local build: using {binary_path}")
    version_output = run_cmd(
        [str(binary_path), "--version"],
        timeout=30,
        env=env_for_subprocess(vex_bin=binary_path),
    ).output.strip()
    match = re.fullmatch(r"vex\s+(.+)", version_output)
    if not match:
        raise TestFailure(f"unexpected --version output from local vex build: {version_output}")
    version = match.group(1).strip()
    return VexReleasePlan(
        source_kind="local-build",
        source_label=f"local build under test at {binary_path}",
        tag_name=f"local-{version}",
        version=version,
        asset_name=binary_path.name,
        download_url="",
        binary_path=binary_path,
    )


def prepare_fresh_vex_release() -> VexReleasePlan:
    REPORT.info("vex release: fetching latest release metadata")
    release = fetch_json(
        VEX_RELEASE_API,
        headers={"Accept": "application/vnd.github+json"},
    )
    tag_name = str(release.get("tag_name") or "").strip()
    if not tag_name:
        raise TestFailure(f"latest vex release metadata from {VEX_RELEASE_API} did not include tag_name")
    assets = release.get("assets")
    if not isinstance(assets, list) or not assets:
        raise TestFailure(f"latest vex release metadata from {VEX_RELEASE_API} did not include assets")

    arch_suffix = vex_release_arch_suffix()
    chosen_asset = None
    for extension in (".tar.xz", ".tar.gz", ""):
        for asset in assets:
            name = str(asset.get("name") or "")
            if ".sha256" in name or arch_suffix not in name:
                continue
            if extension:
                if not name.endswith(extension):
                    continue
            elif any(name.endswith(suffix) for suffix in (".tar.xz", ".tar.gz", ".zip")):
                continue
            chosen_asset = asset
            break
        if chosen_asset is not None:
            break

    if chosen_asset is None:
        raise TestFailure(f"could not find a vex release asset for {arch_suffix} in {tag_name}")

    asset_name = str(chosen_asset.get("name") or "").strip()
    download_url = str(chosen_asset.get("browser_download_url") or "").strip()
    if not asset_name or not download_url:
        raise TestFailure(f"chosen vex release asset for {tag_name} is missing name or browser_download_url")

    VEX_RELEASE_ROOT.mkdir(parents=True, exist_ok=True)
    asset_path = VEX_RELEASE_ROOT / asset_name
    if asset_path.exists():
        asset_path.unlink()
    REPORT.info(f"vex release: downloading {asset_name} (progress below)")
    download_archive_with_curl(download_url, asset_path, label="vex release asset")
    binary_path = extract_vex_release_binary(asset_path)
    binary_path.chmod(0o755)

    return VexReleasePlan(
        source_kind="release",
        source_label=f"fresh release asset {asset_name} from {tag_name}",
        tag_name=tag_name,
        version=strip_v_prefix(tag_name),
        asset_name=asset_name,
        download_url=download_url,
        binary_path=binary_path.resolve(),
    )


def resolve_vex_under_test() -> VexReleasePlan:
    if STRICT_USE_LOCAL_BUILD:
        return resolve_local_build_vex()
    return prepare_fresh_vex_release()


def extract_vex_release_binary(asset_path: Path) -> Path:
    output_path = VEX_RELEASE_ROOT / "vex"
    if output_path.exists():
        output_path.unlink()

    if asset_path.suffixes[-2:] in [[".tar", ".xz"], [".tar", ".gz"]]:
        try:
            with tarfile.open(asset_path, mode="r:*") as archive:
                for member in archive:
                    if not member.isfile() or Path(member.name).name != "vex":
                        continue
                    source = archive.extractfile(member)
                    if source is None:
                        continue
                    with output_path.open("wb") as target:
                        shutil.copyfileobj(source, target)
                    return output_path
        except (OSError, tarfile.TarError) as exc:
            raise TestFailure(f"failed to extract vex binary from {asset_path.name}: {exc}") from exc
        raise TestFailure(f"could not find a vex binary inside {asset_path.name}")

    shutil.copy2(asset_path, output_path)
    return output_path


def parse_help_commands(output: str) -> Dict[str, str]:
    commands: Dict[str, str] = {}
    in_commands = False
    for line in output.splitlines():
        stripped = line.strip()
        if stripped == "Commands:":
            in_commands = True
            continue
        if not in_commands:
            continue
        if stripped == "Options:":
            break
        if not stripped:
            continue
        match = re.match(r"^\s{2,}([a-z][a-z0-9-]*)\s+(.*)$", line)
        if not match:
            continue
        commands[match.group(1)] = match.group(2).strip()
    return commands


def strip_ansi(text: str) -> str:
    return re.sub(r"\x1b\[[0-9;]*m", "", text)


def validate_cli_help_surface(vex_release: VexReleasePlan) -> None:
    REPORT.section("CLI Help")
    version_subject = "downloaded release version" if vex_release.source_kind == "release" else "vex-under-test version"
    version_long = run_cmd([str(VEX_BIN), "--version"], timeout=30)
    version_short = run_cmd([str(VEX_BIN), "-V"], timeout=30)
    REPORT.expect(
        version_long.output.strip() == f"vex {vex_release.version}",
        f"vex --version matches the {version_subject}",
        f"unexpected vex --version output: {version_long.output.strip()}",
    )
    REPORT.expect(
        version_short.output.strip() == version_long.output.strip(),
        "vex -V matches vex --version",
        f"vex -V output mismatch: {version_short.output.strip()} vs {version_long.output.strip()}",
    )

    top_help = run_cmd([str(VEX_BIN), "--help"], timeout=30)
    help_command = run_cmd([str(VEX_BIN), "help"], timeout=30)
    for label, result in [("vex --help", top_help), ("vex help", help_command)]:
        REPORT.expect(
            "A fast version manager for macOS" in result.output,
            f"{label} includes the CLI summary",
            f"{label} missing CLI summary: {result.output.strip()}",
        )
        REPORT.expect(
            "Usage: vex <COMMAND>" in result.output,
            f"{label} includes top-level usage",
            f"{label} missing top-level usage: {result.output.strip()}",
        )
        parsed = parse_help_commands(result.output)
        missing = [command for command in EXPECTED_TOP_LEVEL_COMMANDS if command not in parsed]
        REPORT.expect(
            not missing,
            f"{label} lists every expected top-level command",
            f"{label} missing commands: {', '.join(missing)}",
        )
        mismatched = [
            command
            for command, description in EXPECTED_TOP_LEVEL_COMMANDS.items()
            if command in parsed and description not in parsed[command]
        ]
        REPORT.expect(
            not mismatched,
            f"{label} descriptions match expected command summaries",
            f"{label} summary mismatch for: {', '.join(mismatched)}",
        )
        extra = sorted(set(parsed) - set(EXPECTED_TOP_LEVEL_COMMANDS))
        if extra:
            REPORT.warn(f"{label} exposes additional commands not explicitly covered: {', '.join(extra)}")

    invalid_help = run_cmd([str(VEX_BIN), "help", "--help"], timeout=30, allow_nonzero=True)
    REPORT.expect(
        invalid_help.returncode != 0 and "unrecognized subcommand '--help'" in invalid_help.output,
        "vex help --help fails cleanly with Clap's built-in help-subcommand behavior",
        f"unexpected vex help --help behavior: {invalid_help.output.strip()}",
    )

    for command, snippets in COMMAND_HELP_CHECKS.items():
        validate_help_output(command, snippets)


def validate_help_output(command: str, snippets: Sequence[str]) -> None:
    for invocation in (
        [str(VEX_BIN), command, "--help"],
        [str(VEX_BIN), "help", command],
    ):
        result = run_cmd(invocation, timeout=30)
        label = " ".join(invocation[1:])
        REPORT.expect(
            all(snippet in result.output for snippet in snippets),
            f"{label} exposes the expected help surface",
            f"{label} help output missing expected content: {result.output.strip()}",
        )


def validate_init_and_shells(vex_release: VexReleasePlan) -> None:
    REPORT.section("Init And Env")

    for shell_name, snippets in ENV_HOOK_CHECKS.items():
        env_output = run_cmd([str(VEX_BIN), "env", shell_name], timeout=30)
        REPORT.expect(
            all(snippet in env_output.output for snippet in snippets),
            f"vex env {shell_name} emits the expected shell hook",
            f"vex env {shell_name} output missing expected hook fragments: {env_output.output.strip()}",
        )

    unsupported_env = run_cmd([str(VEX_BIN), "env", "powershell"], timeout=30, allow_nonzero=True)
    REPORT.expect(
        unsupported_env.returncode != 0 and "Unsupported shell" in unsupported_env.output,
        "vex env rejects unsupported shells cleanly",
        f"unexpected vex env powershell behavior: {unsupported_env.output.strip()}",
    )

    dry_run = run_cmd([str(VEX_BIN), "init", "--shell", "zsh", "--dry-run"], timeout=30)
    dry_run_output = strip_ansi(dry_run.output)
    REPORT.expect(
        "Would create" in dry_run_output,
        "vex init --dry-run previews directory creation",
        f"unexpected init --dry-run output: {dry_run.output.strip()}",
    )
    REPORT.expect(
        "Would append to" in dry_run_output and ".zshrc" in dry_run_output,
        "vex init --dry-run previews shell configuration changes",
        f"unexpected init --dry-run shell preview: {dry_run.output.strip()}",
    )
    REPORT.expect(
        not (TEST_HOME / ".vex").joinpath("config.toml").exists(),
        "vex init --dry-run does not create .vex/config.toml",
        "vex init --dry-run modified the filesystem",
    )
    REPORT.expect(
        not (TEST_HOME / ".zshrc").exists(),
        "vex init --dry-run does not create ~/.zshrc",
        "vex init --dry-run created ~/.zshrc unexpectedly",
    )

    init_result = run_cmd([str(VEX_BIN), "init", "--shell", "zsh"], timeout=60)
    init_output = strip_ansi(init_result.output)
    vex_home = TEST_HOME / ".vex"
    for path in [
        vex_home / "cache",
        vex_home / "locks",
        vex_home / "toolchains",
        vex_home / "current",
        vex_home / "bin",
        vex_home / "config.toml",
    ]:
        REPORT.expect(path.exists(), f"{path.relative_to(TEST_HOME)} created by vex init", f"missing init artifact {path}")
    zshrc = TEST_HOME / ".zshrc"
    zshrc_content = zshrc.read_text(errors="replace") if zshrc.exists() else ""
    REPORT.expect(
        "Configured zsh shell integration" in init_output,
        "vex init --shell zsh reports shell integration setup",
        f"unexpected vex init --shell zsh output: {init_result.output.strip()}",
    )
    REPORT.expect(
        '# vex shell integration' in zshrc_content and 'eval "$(vex env zsh)"' in zshrc_content,
        "vex init --shell zsh appends the expected ~/.zshrc hook",
        f"unexpected ~/.zshrc contents after init: {zshrc_content.strip()}",
    )

    repeat_init = run_cmd([str(VEX_BIN), "init", "--shell", "zsh"], timeout=60)
    repeat_init_output = strip_ansi(repeat_init.output)
    REPORT.expect(
        "already configured" in repeat_init_output,
        "vex init detects pre-existing shell integration",
        f"unexpected repeated vex init output: {repeat_init.output.strip()}",
    )

    auto_home = TMP_ROOT / "auto-home"
    if auto_home.exists():
        shutil.rmtree(auto_home, ignore_errors=True)
    auto_home.mkdir(parents=True, exist_ok=True)
    auto_env = env_for_subprocess(home=auto_home, vex_bin=VEX_BIN)
    auto_env["SHELL"] = "/bin/zsh"
    auto_init = run_cmd(
        [str(VEX_BIN), "init", "--shell", "auto"],
        timeout=60,
        env=auto_env,
    )
    auto_init_output = strip_ansi(auto_init.output)
    auto_zshrc = auto_home / ".zshrc"
    auto_zshrc_content = auto_zshrc.read_text(errors="replace") if auto_zshrc.exists() else ""
    REPORT.expect(
        "Configured zsh shell integration" in auto_init_output,
        "vex init --shell auto detects zsh from SHELL",
        f"unexpected vex init --shell auto output: {auto_init.output.strip()}",
    )
    REPORT.expect(
        'eval "$(vex env zsh)"' in auto_zshrc_content,
        "vex init --shell auto writes the detected zsh hook",
        f"unexpected auto-detected ~/.zshrc contents: {auto_zshrc_content.strip()}",
    )
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
        meta={"releases": normalized, "versions": [item["version"] for item in normalized]},
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
        meta={"versions": versions},
    )


def resolve_java(spec: str) -> ToolPlan:
    REPORT.info("Java: fetching Adoptium release metadata")
    try:
        releases = fetch_json("https://api.adoptium.net/v3/info/available_releases")
    except Exception as e:
        # If API fails, use fallback with known LTS versions
        REPORT.warn(f"Java upstream resolution failed: {e}")
        REPORT.warn("Using fallback Java LTS version")
        arch = "aarch64" if os.uname().machine in {"arm64", "aarch64"} else "x64"
        # Use known stable LTS version as fallback
        resolved = "21"
        download_url = resolve_java_download_url(resolved, arch)
        return ToolPlan(
            name="java",
            display_name="Java",
            requested_spec=spec,
            resolved_version=resolved,
            download_url=download_url,
            upstream_bins={},
            install_spec=f"java@{resolved}",
            bin_regex=JAVA_BIN_RE,
            meta={"versions": [resolved], "lts_versions": [resolved]},
        )

    available = sorted(
        (value for value in map(int, releases["available_releases"]) if value > 0),
        reverse=True,
    )
    lts = sorted(
        (value for value in map(int, releases.get("available_lts_releases", [])) if value > 0),
        reverse=True,
    )
    most_recent_lts = releases.get("most_recent_lts")
    if not lts and most_recent_lts is not None:
        most_recent_lts_int = int(most_recent_lts)
        if most_recent_lts_int > 0:
            lts = [most_recent_lts_int]
    arch = "aarch64" if os.uname().machine in {"arm64", "aarch64"} else "x64"

    if spec == "latest":
        if available:
            resolved = str(available[0])
            download_url = resolve_java_download_url(resolved, arch)
        else:
            resolved, download_url = resolve_java_fallback_lts(releases, arch)
    elif spec == "lts":
        if lts:
            resolved = str(lts[0])
            download_url = resolve_java_download_url(resolved, arch)
        else:
            resolved, download_url = resolve_java_fallback_lts(releases, arch)
    else:
        resolved = str(int(spec))
        download_url = resolve_java_download_url(resolved, arch)

    return ToolPlan(
        name="java",
        display_name="Java",
        requested_spec=spec,
        resolved_version=resolved,
        download_url=download_url,
        upstream_bins={},
        install_spec=f"java@{resolved}",
        bin_regex=JAVA_BIN_RE,
        meta={"versions": [str(version) for version in available], "lts_versions": [str(version) for version in lts]},
    )


def resolve_java_download_url(version: str, arch: str) -> str:
    asset_url = (
        f"https://api.adoptium.net/v3/assets/latest/{version}/hotspot"
        f"?architecture={arch}&image_type=jdk&os=mac&vendor=eclipse"
    )
    assets = fetch_json(asset_url)
    if not isinstance(assets, list) or not assets:
        raise TestFailure(f"Adoptium returned no macOS JDK asset for Java {version}")
    return assets[0]["binary"]["package"]["link"]


def resolve_java_fallback_lts(releases: Dict[str, object], arch: str) -> Tuple[str, str]:
    for candidate in JAVA_FALLBACK_LTS_CANDIDATES:
        try:
            download_url = resolve_java_download_url(str(candidate), arch)
            REPORT.warn(
                "Java metadata was incomplete; falling back to probing known LTS releases"
            )
            return str(candidate), download_url
        except TestFailure:
            continue
    raise TestFailure(f"Adoptium returned no LTS releases in metadata: {releases}")


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
        meta={"stable_version": stable_version},
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

    prefix = f"cpython-{resolved}+"
    matching_name = next((asset_name for asset_name in asset_names if asset_name.startswith(prefix)), None)
    if not matching_name:
        raise TestFailure(f"python-build-standalone has no matching asset for {resolved}")
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
        meta={"versions": versions, "release_tag": tag},
    )


def choose_alt_versions(plans: Dict[str, ToolPlan]) -> Dict[str, str]:
    REPORT.section("Alternate Versions")
    alt_versions: Dict[str, str] = {}
    for tool_name in ["node", "go", "java", "rust", "python"]:
        plan = plans[tool_name]
        alt_version = choose_alt_version(plan)
        if alt_version is None:
            raise TestFailure(f"could not determine an alternate version for {plan.display_name}")
        alt_versions[tool_name] = alt_version
        REPORT.ok(
            f"{plan.display_name}: selected alternate version {alt_version} for multi-version switching tests"
        )
    return alt_versions


def choose_alt_version(plan: ToolPlan) -> Optional[str]:
    versions = plan.meta.get("versions")
    if isinstance(versions, list):
        normalized = [str(version) for version in versions if str(version) != plan.resolved_version]
        selected = pick_preferred_alt_version(plan.name, plan.resolved_version, normalized)
        if selected is not None:
            return selected
    if plan.name == "java":
        return choose_alt_java_version(plan)
    if plan.name == "rust":
        return choose_alt_rust_version(plan)
    return None


def pick_preferred_alt_version(tool_name: str, resolved_version: str, versions: Sequence[str]) -> Optional[str]:
    if not versions:
        return None

    if tool_name == "node":
        major = resolved_version.split(".", 1)[0]
        for version in versions:
            if version.startswith(f"{major}."):
                return version

    if tool_name == "python":
        major_minor = ".".join(resolved_version.split(".")[:2])
        for version in versions:
            if version.startswith(f"{major_minor}."):
                return version
        resolved_major = resolved_version.split(".", 1)[0]
        for version in versions:
            if version.startswith(f"{resolved_major}."):
                return version

    if tool_name == "go":
        major = resolved_version.split(".", 1)[0]
        for version in versions:
            if version.startswith(f"{major}."):
                return version

    return versions[0]


def choose_alt_rust_version(plan: ToolPlan) -> Optional[str]:
    parts = [int(part) for part in plan.resolved_version.split(".")]
    if len(parts) < 3:
        return None
    major, minor, patch = parts[:3]
    target = "aarch64-apple-darwin" if os.uname().machine in {"arm64", "aarch64"} else "x86_64-apple-darwin"
    candidates: List[str] = []
    if patch > 0:
        candidates.append(f"{major}.{minor}.{patch - 1}")
    for minor_delta in range(1, 7):
        candidate_minor = minor - minor_delta
        if candidate_minor < 0:
            break
        candidates.append(f"{major}.{candidate_minor}.1")
        candidates.append(f"{major}.{candidate_minor}.0")

    seen: set[str] = set()
    for candidate in candidates:
        if candidate == plan.resolved_version or candidate in seen:
            continue
        seen.add(candidate)
        url = f"https://static.rust-lang.org/dist/rust-{candidate}-{target}.tar.gz"
        if official_url_exists(url):
            return candidate
    return None


def choose_alt_java_version(plan: ToolPlan) -> Optional[str]:
    arch = "aarch64" if os.uname().machine in {"arm64", "aarch64"} else "x64"
    for candidate in JAVA_FALLBACK_LTS_CANDIDATES:
        version = str(candidate)
        if version == plan.resolved_version:
            continue
        try:
            resolve_java_download_url(version, arch)
            return version
        except TestFailure:
            continue
    return None


def official_url_exists(url: str) -> bool:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT}, method="HEAD")
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return 200 <= response.status < 400
    except Exception:
        if shutil.which("curl") is None:
            return False
        result = run_cmd(
            [
                "curl",
                "-L",
                "--head",
                "--silent",
                "--show-error",
                "--fail",
                "--output",
                "/dev/null",
                url,
            ],
            timeout=60,
            allow_nonzero=True,
        )
        return result.returncode == 0


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
        if bin_name == "serialver":
            return [
                Probe(["--help"], [r"use:\s+serialver|用法:\s*serialver"], True),
                Probe(["-help"], [r"use:\s+serialver|用法:\s*serialver"], True),
                Probe(["-?"], [r"use:\s+serialver|用法:\s*serialver"], True),
                Probe(["-version"], [r"use:\s+serialver|用法:\s*serialver"], True),
            ]
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


def version_probe_command(tool_name: str) -> List[str]:
    if tool_name == "node":
        return ["node", "--version"]
    if tool_name == "go":
        return ["go", "version"]
    if tool_name == "java":
        return ["java", "-version"]
    if tool_name == "rust":
        return ["rustc", "--version"]
    if tool_name == "python":
        return ["python", "--version"]
    raise TestFailure(f"no version probe command defined for {tool_name}")


def first_output_line(output: str) -> str:
    for line in output.splitlines():
        stripped = line.strip()
        if stripped:
            return stripped
    return output.strip()


def version_output_matches(tool_name: str, expected_version: str, output: str) -> bool:
    line = first_output_line(output)
    if tool_name == "node":
        return line == f"v{expected_version}"
    if tool_name == "go":
        return f"go{expected_version}" in line
    if tool_name == "java":
        return bool(
            re.search(rf'version "{re.escape(expected_version)}(?:[."\s]|$)', line)
            or re.search(rf"openjdk {re.escape(expected_version)}(?:[.\s]|$)", line)
        )
    if tool_name == "rust":
        return bool(re.search(rf"^rustc {re.escape(expected_version)}(?:\s|$)", line))
    if tool_name == "python":
        return line == f"Python {expected_version}"
    return expected_version in line


def assert_tool_command_version(tool_name: str, expected_version: str, *, cwd: Optional[Path] = None) -> None:
    result = run_cmd(version_probe_command(tool_name), cwd=cwd, timeout=30)
    REPORT.expect(
        version_output_matches(tool_name, expected_version, result.output),
        f"{tool_name} command resolves to version {expected_version}",
        f"{tool_name} command version mismatch for {expected_version}: {result.output.strip()}",
    )


def validate_manual_multiversion_switching(plans: Dict[str, ToolPlan], alt_versions: Dict[str, str]) -> None:
    REPORT.section("Manual Multi-Version Switching")
    for tool_name in ["node", "go", "java", "rust", "python"]:
        plan = plans[tool_name]
        alt_version = alt_versions[tool_name]
        REPORT.info(
            f"{plan.display_name}: installing alternate version {alt_version} for explicit switch validation (progress below)"
        )
        run_cmd_live([str(VEX_BIN), "install", f"{tool_name}@{alt_version}"], timeout=1800)
        run_cmd([str(VEX_BIN), "use", f"{tool_name}@{alt_version}"], timeout=180)
        current_output = run_cmd([str(VEX_BIN), "current"], timeout=60).output
        REPORT.expect(
            tool_name in current_output and alt_version in current_output,
            f"vex current switches {tool_name} to alternate version {alt_version}",
            f"vex current missing alternate {tool_name}@{alt_version}: {current_output.strip()}",
        )
        assert_tool_command_version(tool_name, alt_version)

        run_cmd([str(VEX_BIN), "use", plan.install_spec], timeout=180)
        current_output = run_cmd([str(VEX_BIN), "current"], timeout=60).output
        REPORT.expect(
            tool_name in current_output and plan.resolved_version in current_output,
            f"vex current switches {tool_name} back to project target {plan.resolved_version}",
            f"vex current missing restored {tool_name}@{plan.resolved_version}: {current_output.strip()}",
        )
        assert_tool_command_version(tool_name, plan.resolved_version)


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


def validate_project_behavior(plans: Dict[str, ToolPlan], alt_versions: Dict[str, str]) -> None:
    REPORT.section("Project directory detection")
    workspace = TMP_ROOT / "workspace"
    project = PROJECT_ROOT
    workspace.mkdir(parents=True, exist_ok=True)
    project.mkdir(parents=True, exist_ok=True)

    for tool_name in ["node", "go", "java", "rust", "python"]:
        alt_version = alt_versions[tool_name]
        REPORT.info(f"Project test: setting global {tool_name}@{alt_version}")
        run_cmd([str(VEX_BIN), "global", f"{tool_name}@{alt_version}"], timeout=120)
        run_cmd([str(VEX_BIN), "use", f"{tool_name}@{alt_version}"], timeout=180)
        REPORT.ok(f"global default pinned to {tool_name}@{alt_version}")

    tool_versions = "\n".join(
        f"{name} {plans[name].resolved_version}" for name in ["node", "go", "java", "rust", "python"]
    ) + "\n"
    write_text(project / ".tool-versions", tool_versions)
    global_versions = "\n".join(
        f"{name} {alt_versions[name]}" for name in ["node", "go", "java", "rust", "python"]
    ) + "\n"
    global_versions_path = TEST_HOME / ".vex" / "tool-versions"
    global_versions_content = global_versions_path.read_text(errors="replace") if global_versions_path.exists() else ""
    expected_global_versions = parse_tool_versions(global_versions)
    actual_global_versions = parse_tool_versions(global_versions_content)
    REPORT.expect(
        actual_global_versions == expected_global_versions,
        "global tool-versions file contains every alternate version pin (order-independent)",
        f"unexpected global tool-versions content: {global_versions_content.strip()}",
    )

    global_current = run_cmd([str(VEX_BIN), "current"], cwd=workspace, timeout=60).output
    REPORT.expect("Global default" in global_current, "outside project shows Global default", f"unexpected global current output: {global_current.strip()}")
    for tool_name in ["node", "go", "java", "rust", "python"]:
        alt_version = alt_versions[tool_name]
        REPORT.expect(
            tool_name in global_current and alt_version in global_current,
            f"outside project current includes global {tool_name}@{alt_version}",
            f"outside project current missing {tool_name}@{alt_version}: {global_current.strip()}",
        )
        assert_tool_command_version(tool_name, alt_version, cwd=workspace)

    run_cmd([str(VEX_BIN), "use", "--auto"], cwd=project, timeout=120)
    project_current = run_cmd([str(VEX_BIN), "current"], cwd=project, timeout=60).output
    REPORT.expect("Project override" in project_current, "inside project shows Project override", f"unexpected project current output: {project_current.strip()}")
    for name in ["node", "go", "java", "rust", "python"]:
        REPORT.expect(
            name in project_current and plans[name].resolved_version in project_current,
            f"project current includes {name}@{plans[name].resolved_version}",
            f"project current missing {name}@{plans[name].resolved_version}",
        )
        assert_tool_command_version(name, plans[name].resolved_version, cwd=project)

    zsh_script = textwrap.dedent(
        f"""
        export HOME={shell_quote(str(TEST_HOME))}
        export PATH={shell_quote(str(VEX_BIN.parent))}:$PATH
        unset VIRTUAL_ENV
        eval "$({shell_quote(str(VEX_BIN))} env zsh)"
        cd {shell_quote(str(workspace))}
        echo "OUTSIDE_NODE=$(node --version 2>&1 | head -n 1)"
        echo "OUTSIDE_GO=$(go version 2>&1 | head -n 1)"
        echo "OUTSIDE_JAVA=$(java -version 2>&1 | head -n 1)"
        echo "OUTSIDE_RUST=$(rustc --version 2>&1 | head -n 1)"
        echo "OUTSIDE_PYTHON=$(python --version 2>&1 | head -n 1)"
        echo "OUTSIDE_VIRTUAL_ENV=${{VIRTUAL_ENV:-}}"
        cd {shell_quote(str(project))}
        echo "INSIDE_NODE=$(node --version 2>&1 | head -n 1)"
        echo "INSIDE_GO=$(go version 2>&1 | head -n 1)"
        echo "INSIDE_JAVA=$(java -version 2>&1 | head -n 1)"
        echo "INSIDE_RUST=$(rustc --version 2>&1 | head -n 1)"
        echo "INSIDE_PYTHON=$(python --version 2>&1 | head -n 1)"
        echo "INSIDE_VIRTUAL_ENV=${{VIRTUAL_ENV:-}}"
        {shell_quote(str(VEX_BIN))} current
        cd {shell_quote(str(workspace))}
        echo "OUTSIDE_AGAIN_NODE=$(node --version 2>&1 | head -n 1)"
        echo "OUTSIDE_AGAIN_GO=$(go version 2>&1 | head -n 1)"
        echo "OUTSIDE_AGAIN_JAVA=$(java -version 2>&1 | head -n 1)"
        echo "OUTSIDE_AGAIN_RUST=$(rustc --version 2>&1 | head -n 1)"
        echo "OUTSIDE_AGAIN_PYTHON=$(python --version 2>&1 | head -n 1)"
        echo "OUTSIDE_AGAIN_VIRTUAL_ENV=${{VIRTUAL_ENV:-}}"
        """
    ).strip()
    hook_result = run_cmd(["zsh", "-lc", zsh_script], timeout=120, allow_nonzero=False)

    hook_markers: Dict[str, str] = {}
    for line in hook_result.output.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        if key.startswith(("OUTSIDE_", "INSIDE_", "OUTSIDE_AGAIN_")):
            hook_markers[key.strip()] = value.strip()

    for tool_name in ["node", "go", "java", "rust", "python"]:
        outside_key = f"OUTSIDE_{tool_name.upper()}"
        inside_key = f"INSIDE_{tool_name.upper()}"
        outside_again_key = f"OUTSIDE_AGAIN_{tool_name.upper()}"
        REPORT.expect(
            outside_key in hook_markers and version_output_matches(tool_name, alt_versions[tool_name], hook_markers[outside_key]),
            f"cd hook keeps global {tool_name}@{alt_versions[tool_name]} outside the project",
            f"{tool_name} outside-project hook mismatch: {hook_result.output.strip()}",
        )
        REPORT.expect(
            inside_key in hook_markers and version_output_matches(tool_name, plans[tool_name].resolved_version, hook_markers[inside_key]),
            f"cd hook switches {tool_name} to project version {plans[tool_name].resolved_version}",
            f"{tool_name} inside-project hook mismatch: {hook_result.output.strip()}",
        )
        REPORT.expect(
            outside_again_key in hook_markers and version_output_matches(tool_name, alt_versions[tool_name], hook_markers[outside_again_key]),
            f"cd hook restores global {tool_name}@{alt_versions[tool_name]} after leaving the project",
            f"{tool_name} outside-again hook mismatch: {hook_result.output.strip()}",
        )

    expected_venv = (project / ".venv").resolve()
    inside_venv = hook_markers.get("INSIDE_VIRTUAL_ENV", "")
    outside_venv = hook_markers.get("OUTSIDE_VIRTUAL_ENV", "")
    outside_again_venv = hook_markers.get("OUTSIDE_AGAIN_VIRTUAL_ENV", "")
    actual_venv = Path(inside_venv).resolve() if inside_venv else None
    REPORT.expect(
        actual_venv == expected_venv,
        "cd hook auto-activates project .venv",
        f"VIRTUAL_ENV not auto-activated: {hook_result.output.strip()}",
    )
    REPORT.expect(
        outside_venv == "",
        "outside project there is no inherited VIRTUAL_ENV before entering the project",
        f"VIRTUAL_ENV leaked before entering the project: {hook_result.output.strip()}",
    )
    REPORT.expect(
        outside_again_venv == "",
        "leaving the project deactivates the auto-activated .venv",
        f"VIRTUAL_ENV remained active after leaving the project: {hook_result.output.strip()}",
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


def parse_tool_versions(content: str) -> Dict[str, str]:
    parsed: Dict[str, str] = {}
    for raw_line in content.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if len(parts) < 2:
            continue
        parsed[parts[0]] = parts[1]
    return parsed


def shell_quote(value: str) -> str:
    return "'" + value.replace("'", "'\"'\"'") + "'"


if __name__ == "__main__":
    raise SystemExit(main())
